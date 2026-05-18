use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{Message, Project, Role, SearchResult, Session, SessionMetadata};
use crate::provider::Provider;
use crate::search::SearchOptions;

pub struct CursorProvider {
    base_path: Option<PathBuf>,
}

impl CursorProvider {
    pub fn new() -> Self {
        Self {
            base_path: get_base_path(),
        }
    }

    fn global_db(&self) -> Option<PathBuf> {
        self.base_path
            .as_ref()
            .map(|b| b.join("globalStorage/state.vscdb"))
    }

    fn workspace_storage_dir(&self) -> Option<PathBuf> {
        self.base_path.as_ref().map(|b| b.join("workspaceStorage"))
    }
}

fn get_base_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    #[cfg(target_os = "macos")]
    let base = home.join("Library/Application Support/Cursor/User");

    #[cfg(target_os = "linux")]
    let base = home.join(".config/Cursor/User");

    #[cfg(target_os = "windows")]
    let base = home.join("AppData/Roaming/Cursor/User");

    if base.is_dir() {
        Some(base)
    } else {
        None
    }
}

impl Provider for CursorProvider {
    fn id(&self) -> &str {
        "cursor"
    }

    fn display_name(&self) -> &str {
        "Cursor"
    }

    fn is_available(&self) -> bool {
        self.global_db()
            .map(|p| p.is_file())
            .unwrap_or(false)
    }

    fn scan_projects(&self) -> Result<Vec<Project>> {
        let ws_dir = match self.workspace_storage_dir() {
            Some(d) if d.is_dir() => d,
            _ => return Ok(Vec::new()),
        };

        let mut projects = Vec::new();

        let entries = fs::read_dir(&ws_dir)?;
        for entry in entries.flatten() {
            let ws_path = entry.path();
            if !ws_path.is_dir() || ws_path.is_symlink() {
                continue;
            }

            let workspace_json = ws_path.join("workspace.json");
            let project_folder = match read_workspace_folder(&workspace_json) {
                Some(f) => f,
                None => continue,
            };

            let ws_db_path = ws_path.join("state.vscdb");
            let composers = match read_workspace_composers(&ws_db_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let active: Vec<&serde_json::Value> = composers
                .iter()
                .filter(|c| {
                    !c.get("isArchived")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false)
                })
                .collect();

            if active.is_empty() {
                continue;
            }

            let session_count = active.len();
            let last_ms = active
                .iter()
                .filter_map(|c| c.get("lastUpdatedAt").and_then(serde_json::Value::as_f64))
                .fold(0.0f64, f64::max);

            projects.push(Project {
                provider: "cursor".to_string(),
                name: project_folder.clone(),
                path: ws_path.to_string_lossy().to_string(),
                session_count,
                last_modified: ms_to_rfc3339(last_ms as u64),
            });
        }

        projects.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        Ok(projects)
    }

    fn list_sessions(&self, project: &Project) -> Result<Vec<Session>> {
        let ws_path = PathBuf::from(&project.path);
        let ws_db_path = ws_path.join("state.vscdb");
        let composers = read_workspace_composers(&ws_db_path)?;

        let mut sessions: Vec<Session> = composers
            .iter()
            .filter_map(|c| {
                let id = c.get("composerId").and_then(serde_json::Value::as_str)?;
                let name = c
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("Untitled");
                let created = c
                    .get("createdAt")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0) as u64;
                let updated = c
                    .get("lastUpdatedAt")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0) as u64;
                let is_archived = c
                    .get("isArchived")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                let mode = c
                    .get("unifiedMode")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("chat");

                if is_archived {
                    return None;
                }

                let mut tools_used = Vec::new();
                if mode == "agent" {
                    tools_used.push("agent-mode".to_string());
                }

                let metadata = if tools_used.is_empty() {
                    None
                } else {
                    Some(SessionMetadata {
                        tools_used,
                        ..Default::default()
                    })
                };

                Some(Session {
                    provider: "cursor".to_string(),
                    id: id.to_string(),
                    file_path: format!("cursor://{id}"),
                    project_name: project.name.clone(),
                    message_count: 0,
                    first_time: ms_to_rfc3339(created),
                    last_time: ms_to_rfc3339(updated),
                    summary: Some(name.to_string()),
                    metadata,
                })
            })
            .collect();

        sessions.sort_by(|a, b| b.last_time.cmp(&a.last_time));
        Ok(sessions)
    }

    fn load_messages(&self, session: &Session) -> Result<Vec<Message>> {
        let composer_id = session
            .file_path
            .strip_prefix("cursor://")
            .unwrap_or(&session.id);

        let global_db_path = self
            .global_db()
            .context("Cursor global database not found")?;

        load_composer_messages(&global_db_path, composer_id)
    }

    fn search(&self, opts: &SearchOptions) -> Result<Vec<SearchResult>> {
        let projects = self.scan_projects()?;
        let mut entries = Vec::new();

        for project in &projects {
            let sessions = self.list_sessions(project)?;
            for session in sessions {
                let messages = self.load_messages(&session)?;
                if !messages.is_empty() {
                    entries.push((session, messages, "cursor".to_string()));
                }
            }
        }

        Ok(crate::search::rank_and_search(entries, opts))
    }
}

fn load_composer_messages(global_db_path: &Path, composer_id: &str) -> Result<Vec<Message>> {
    let conn = open_readonly(global_db_path)?;

    let composer_key = format!("composerData:{composer_id}");
    let composer_data: String = conn
        .query_row(
            "SELECT value FROM cursorDiskKV WHERE key = ?1",
            [&composer_key],
            |row| row.get(0),
        )
        .context("Composer data not found")?;

    let composer: serde_json::Value = parse_cursor_json(&composer_data)?;

    let headers = match composer
        .get("fullConversationHeadersOnly")
        .and_then(serde_json::Value::as_array)
    {
        Some(h) => h,
        None => return Ok(Vec::new()),
    };

    let prefix = format!("bubbleId:{composer_id}:");
    let pattern = format!("{prefix}%");
    let mut stmt = conn.prepare("SELECT key, value FROM cursorDiskKV WHERE key LIKE ?1")?;
    let mut bubble_map: HashMap<String, String> = HashMap::new();
    let rows = stmt.query_map([&pattern], |row| {
        let key: String = row.get(0)?;
        let value: String = row.get(1)?;
        Ok((key, value))
    })?;
    for row in rows.flatten() {
        let (key, value) = row;
        if let Some(bid) = key.strip_prefix(&prefix) {
            bubble_map.insert(bid.to_string(), value);
        }
    }

    let mut messages = Vec::with_capacity(headers.len());

    for header in headers {
        let bubble_id = match header.get("bubbleId").and_then(serde_json::Value::as_str) {
            Some(id) => id,
            None => continue,
        };
        let bubble_type = header
            .get("type")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        let bubble_data = match bubble_map.get(bubble_id) {
            Some(d) => d,
            None => continue,
        };

        let bubble: serde_json::Value = match parse_cursor_json(bubble_data) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let ts = extract_bubble_timestamp(&bubble);

        match bubble_type {
            1 => {
                let text = bubble
                    .get("text")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                if !text.is_empty() {
                    messages.push(Message {
                        role: Role::User,
                        timestamp: ts,
                        text: text.to_string(),
                        tool_name: None,
                        tool_input: None,
                        tool_output: None,
                        model: None,
                        thinking: None,
                    });
                }
            }
            2 => {
                convert_assistant_bubble(&bubble, &ts, &mut messages);
            }
            _ => {}
        }
    }

    Ok(messages)
}

fn convert_assistant_bubble(bubble: &serde_json::Value, ts: &str, messages: &mut Vec<Message>) {
    let text = bubble
        .get("text")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    if let Some(tool_data) = bubble.get("toolFormerData") {
        let tool_name = tool_data
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let mapped = map_cursor_tool_name(tool_name);
        let raw_args = tool_data
            .get("rawArgs")
            .and_then(serde_json::Value::as_str)
            .map(String::from);
        let status = tool_data
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("completed");

        messages.push(Message {
            role: Role::Tool,
            timestamp: ts.to_string(),
            text: String::new(),
            tool_name: Some(mapped.to_string()),
            tool_input: raw_args,
            tool_output: if !text.is_empty() {
                Some(text.to_string())
            } else {
                None
            },
            model: None,
            thinking: if status == "error" || status == "rejected" {
                Some(format!("[{status}]"))
            } else {
                None
            },
        });
        return;
    }

    if text.is_empty() {
        return;
    }

    let is_thought = bubble
        .get("isThought")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    if is_thought {
        messages.push(Message {
            role: Role::Assistant,
            timestamp: ts.to_string(),
            text: String::new(),
            tool_name: None,
            tool_input: None,
            tool_output: None,
            model: None,
            thinking: Some(text.to_string()),
        });
    } else {
        messages.push(Message {
            role: Role::Assistant,
            timestamp: ts.to_string(),
            text: text.to_string(),
            tool_name: None,
            tool_input: None,
            tool_output: None,
            model: None,
            thinking: None,
        });
    }
}

fn open_readonly(path: &Path) -> Result<Connection> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("Failed to open Cursor DB: {}", path.display()))?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(conn)
}

fn read_workspace_folder(workspace_json: &Path) -> Option<String> {
    let data = fs::read_to_string(workspace_json).ok()?;
    let json: serde_json::Value = serde_json::from_str(&data).ok()?;
    let folder = json.get("folder").and_then(serde_json::Value::as_str)?;
    folder.strip_prefix("file://").map(|s| {
        let path = if s.len() > 2 && s.as_bytes()[2] == b':' {
            &s[1..]
        } else {
            s
        };
        percent_decode(path)
    })
}

fn read_workspace_composers(ws_db_path: &Path) -> Result<Vec<serde_json::Value>> {
    if !ws_db_path.is_file() {
        return Ok(Vec::new());
    }

    let conn = open_readonly(ws_db_path)?;

    let data: Result<String, _> = conn.query_row(
        "SELECT value FROM ItemTable WHERE key = 'composer.composerData'",
        [],
        |row| row.get(0),
    );

    let data = match data {
        Ok(d) => d,
        Err(_) => return Ok(Vec::new()),
    };

    let json: serde_json::Value = parse_cursor_json(&data)?;

    Ok(json
        .get("allComposers")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default())
}

fn parse_cursor_json(data: &str) -> Result<serde_json::Value> {
    let sanitized: String = data
        .chars()
        .map(|c| {
            if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
                ' '
            } else {
                c
            }
        })
        .collect();

    serde_json::from_str(&sanitized).context("Failed to parse Cursor JSON")
}

fn percent_decode(input: &str) -> String {
    let mut buf = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                buf.push(byte);
                i += 3;
                continue;
            }
        }
        buf.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(buf).unwrap_or_else(|_| input.to_string())
}

fn extract_bubble_timestamp(bubble: &serde_json::Value) -> String {
    bubble
        .get("createdAt")
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .or_else(|| {
            bubble
                .get("timestamp")
                .and_then(serde_json::Value::as_str)
                .filter(|s| !s.is_empty())
                .map(String::from)
        })
        .or_else(|| {
            bubble
                .get("createdAt")
                .and_then(serde_json::Value::as_f64)
                .map(|ms| ms_to_rfc3339(ms as u64))
        })
        .or_else(|| {
            bubble
                .get("timestamp")
                .and_then(serde_json::Value::as_f64)
                .map(|ms| ms_to_rfc3339(ms as u64))
        })
        .unwrap_or_default()
}

fn ms_to_rfc3339(ms: u64) -> String {
    use chrono::{TimeZone, Utc};
    let secs = (ms / 1000) as i64;
    let nanos = ((ms % 1000) * 1_000_000) as u32;
    match Utc.timestamp_opt(secs, nanos) {
        chrono::LocalResult::Single(dt) => dt.to_rfc3339(),
        _ => String::new(),
    }
}

fn map_cursor_tool_name(name: &str) -> &str {
    match name {
        "read_file" | "view_file" => "Read",
        "write_to_file" | "create_file" | "edit_file" => "Write",
        "execute_command" | "run_terminal_cmd" => "Bash",
        "list_directory" | "list_dir" => "Glob",
        "search_files" | "codebase_search" | "grep_search" => "Grep",
        "web_search" => "WebSearch",
        "web_fetch" | "fetch_url" => "WebFetch",
        _ => name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cursor_json_with_control_chars() {
        let data = "{\"text\": \"hello\x01world\"}";
        let result = parse_cursor_json(data).unwrap();
        assert!(result["text"].as_str().unwrap().contains("hello"));
    }

    #[test]
    fn test_map_cursor_tool_names() {
        assert_eq!(map_cursor_tool_name("read_file"), "Read");
        assert_eq!(map_cursor_tool_name("execute_command"), "Bash");
        assert_eq!(map_cursor_tool_name("codebase_search"), "Grep");
        assert_eq!(map_cursor_tool_name("write_to_file"), "Write");
        assert_eq!(map_cursor_tool_name("unknown_tool"), "unknown_tool");
    }

    #[test]
    fn test_ms_to_rfc3339() {
        let result = ms_to_rfc3339(1700000000000);
        assert!(result.starts_with("2023-11-14T"));
    }

    #[test]
    fn test_percent_decode() {
        assert_eq!(percent_decode("/path/to/my%20project"), "/path/to/my project");
        assert_eq!(percent_decode("/normal/path"), "/normal/path");
    }

    #[test]
    fn test_extract_bubble_timestamp_string() {
        let bubble = serde_json::json!({
            "createdAt": "2026-01-24T09:45:46.113Z"
        });
        assert_eq!(
            extract_bubble_timestamp(&bubble),
            "2026-01-24T09:45:46.113Z"
        );
    }

    #[test]
    fn test_extract_bubble_timestamp_numeric() {
        let bubble = serde_json::json!({
            "createdAt": 1700000000000_u64
        });
        let ts = extract_bubble_timestamp(&bubble);
        assert!(ts.starts_with("2023-11-14T"));
    }

    #[test]
    fn test_extract_bubble_timestamp_fallback() {
        let bubble = serde_json::json!({
            "timestamp": "2026-01-24T10:00:00Z"
        });
        assert_eq!(
            extract_bubble_timestamp(&bubble),
            "2026-01-24T10:00:00Z"
        );
    }
}
