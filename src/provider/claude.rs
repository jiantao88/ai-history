use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::model::{Message, Project, Role, SearchResult, Session};
use crate::parse;
use crate::provider::Provider;

pub struct ClaudeProvider {
    base_path: PathBuf,
}

impl ClaudeProvider {
    pub fn new() -> Self {
        let base = dirs::home_dir()
            .map(|h| h.join(".claude"))
            .unwrap_or_default();
        Self { base_path: base }
    }

    fn projects_dir(&self) -> PathBuf {
        self.base_path.join("projects")
    }
}

impl Provider for ClaudeProvider {
    fn id(&self) -> &str {
        "claude"
    }

    fn display_name(&self) -> &str {
        "Claude Code"
    }

    fn is_available(&self) -> bool {
        let projects = self.projects_dir();
        projects.exists() && projects.is_dir()
    }

    fn scan_projects(&self) -> Result<Vec<Project>> {
        let projects_dir = self.projects_dir();
        let mut projects = Vec::new();

        let entries: Vec<_> = std::fs::read_dir(&projects_dir)
            .context("Failed to read Claude projects directory")?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .collect();

        for entry in entries {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            let dir_path = entry.path();

            let jsonl_files = count_jsonl_files(&dir_path);
            if jsonl_files == 0 {
                continue;
            }

            let last_modified = newest_jsonl_mtime(&dir_path)
                .unwrap_or_default();

            let actual_path = decode_project_path(&dir_name);

            projects.push(Project {
                provider: "claude".to_string(),
                name: actual_path.clone(),
                path: dir_path.to_string_lossy().to_string(),
                session_count: jsonl_files,
                last_modified,
            });
        }

        projects.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        Ok(projects)
    }

    fn list_sessions(&self, project: &Project) -> Result<Vec<Session>> {
        let dir = Path::new(&project.path);
        let mut sessions = Vec::new();

        for entry in WalkDir::new(dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                if let Some(session) = load_session_metadata(path, &project.name)? {
                    sessions.push(session);
                }
            }
        }

        sessions.sort_by(|a, b| b.last_time.cmp(&a.last_time));
        Ok(sessions)
    }

    fn load_messages(&self, session: &Session) -> Result<Vec<Message>> {
        let path = Path::new(&session.file_path);
        load_claude_messages(path)
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let query_lower = query.to_lowercase();
        let projects = self.scan_projects()?;
        let mut results = Vec::new();

        'outer: for project in &projects {
            let sessions = self.list_sessions(project)?;
            for session in &sessions {
                let messages = self.load_messages(session)?;
                for msg in &messages {
                    if msg.text.to_lowercase().contains(&query_lower) {
                        results.push(SearchResult {
                            message: msg.clone(),
                            session_id: session.id.clone(),
                            project_name: project.name.clone(),
                            provider: "claude".to_string(),
                        });
                        if results.len() >= limit {
                            break 'outer;
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}

fn count_jsonl_files(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "jsonl")
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

fn newest_jsonl_mtime(dir: &Path) -> Option<String> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "jsonl")
                .unwrap_or(false)
        })
        .filter_map(|e| e.metadata().ok()?.modified().ok())
        .max()
        .map(|t| {
            let dt: chrono::DateTime<chrono::Utc> = t.into();
            dt.to_rfc3339()
        })
}

fn decode_project_path(encoded: &str) -> String {
    let stripped = encoded.strip_prefix('-').unwrap_or(encoded);
    decode_path_with_prefix("", stripped, 0).unwrap_or_else(|| {
        let replaced = encoded.replace('-', "/");
        if replaced.starts_with('/') {
            replaced
        } else {
            format!("/{replaced}")
        }
    })
}

fn decode_path_with_prefix(prefix: &str, remaining: &str, depth: usize) -> Option<String> {
    if depth > 20 || remaining.is_empty() {
        return Some(prefix.to_string());
    }

    let parts: Vec<&str> = remaining.split('-').collect();
    if parts.is_empty() {
        return Some(prefix.to_string());
    }

    for i in (1..=parts.len()).rev() {
        let segment = parts[..i].join("-");
        let candidate = format!("{prefix}/{segment}");
        let candidate_path = std::path::Path::new(&candidate);

        if candidate_path.symlink_metadata().is_ok() {
            if i == parts.len() {
                return Some(candidate);
            }
            let rest = parts[i..].join("-");
            if let Some(result) = decode_path_with_prefix(&candidate, &rest, depth + 1) {
                return Some(result);
            }
        }
    }

    None
}

const SKIP_TYPES: &[&str] = &[
    "progress",
    "queue-operation",
    "file-history-snapshot",
    "last-prompt",
    "pr-link",
];

const SKIP_SUBTYPES: &[&str] = &[
    "stop_hook_summary",
    "turn_duration",
    "microcompact_boundary",
];

fn should_skip_message(entry: &serde_json::Value) -> bool {
    if entry.get("isMeta").and_then(|v| v.as_bool()).unwrap_or(false) {
        return true;
    }

    if let Some(msg_type) = entry.get("type").and_then(|t| t.as_str()) {
        if SKIP_TYPES.contains(&msg_type) {
            return true;
        }

        if msg_type == "system" {
            if let Some(message) = entry.get("message") {
                if let Some(subtype) = message.get("subtype").and_then(|s| s.as_str()) {
                    if SKIP_SUBTYPES.contains(&subtype) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

fn load_session_metadata(path: &Path, project_name: &str) -> Result<Option<Session>> {
    let mmap = parse::mmap_file(path)?;
    let ranges = parse::find_line_ranges(&mmap);

    if ranges.is_empty() {
        return Ok(None);
    }

    let mut session_id = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut first_time = String::new();
    let mut last_time = String::new();
    let mut message_count = 0usize;
    let mut summary: Option<String> = None;
    let mut first_user_text: Option<String> = None;

    let scan_limit = ranges.len().min(100);
    for &(start, end) in &ranges[..scan_limit] {
        let line = &mmap[start..end];
        let Some(entry) = parse::parse_jsonl_line(line) else {
            continue;
        };

        if should_skip_message(&entry) {
            continue;
        }

        if let Some(ts) = entry.get("timestamp").and_then(|t| t.as_str()) {
            if first_time.is_empty() {
                first_time = ts.to_string();
            }
            last_time = ts.to_string();
        }

        if let Some(sid) = entry.get("sessionId").and_then(|s| s.as_str()) {
            if session_id == path.file_stem().unwrap().to_string_lossy() {
                session_id = sid.to_string();
            }
        }

        let msg_type = entry.get("type").and_then(|t| t.as_str()).unwrap_or("");

        if msg_type == "summary" {
            if let Some(s) = entry.get("summary").and_then(|s| s.as_str()) {
                summary = Some(s.to_string());
            }
        }

        if msg_type == "user" || msg_type == "assistant" {
            message_count += 1;
        }

        if msg_type == "user" && first_user_text.is_none() {
            if let Some(message) = entry.get("message") {
                if let Some(content) = message.get("content") {
                    let text = parse::extract_text_from_content(content);
                    if !text.is_empty() {
                        first_user_text = Some(truncate_string(&text, 100));
                    }
                }
            }
        }
    }

    if scan_limit < ranges.len() {
        for &(start, end) in &ranges[scan_limit..] {
            let line = &mmap[start..end];
            if line.windows(6).any(|w| w == b"\"user\"" || w == b"\"assi\"") {
                message_count += 1;
            }
            if let Some(ts_start) = find_timestamp_fast(line) {
                let ts = &line[ts_start..];
                if let Some(end_pos) = ts.iter().position(|&b| b == b'"') {
                    let ts_str = std::str::from_utf8(&ts[..end_pos]).unwrap_or("");
                    if ts_str > last_time.as_str() {
                        last_time = ts_str.to_string();
                    }
                }
            }
        }
    }

    if first_time.is_empty() {
        return Ok(None);
    }

    let final_summary = summary.or(first_user_text);

    Ok(Some(Session {
        provider: "claude".to_string(),
        id: session_id,
        file_path: path.to_string_lossy().to_string(),
        project_name: project_name.to_string(),
        message_count,
        first_time,
        last_time,
        summary: final_summary,
    }))
}

fn find_timestamp_fast(line: &[u8]) -> Option<usize> {
    let needle = b"\"timestamp\":\"";
    line.windows(needle.len())
        .position(|w| w == needle)
        .map(|pos| pos + needle.len())
}

fn load_claude_messages(path: &Path) -> Result<Vec<Message>> {
    let mmap = parse::mmap_file(path)?;
    let ranges = parse::find_line_ranges(&mmap);
    let mut messages = Vec::new();

    for &(start, end) in &ranges {
        let line = &mmap[start..end];
        let Some(entry) = parse::parse_jsonl_line(line) else {
            continue;
        };

        if should_skip_message(&entry) {
            continue;
        }

        let msg_type = entry.get("type").and_then(|t| t.as_str()).unwrap_or("");

        if msg_type == "summary" {
            continue;
        }

        let is_sidechain = entry
            .get("isSidechain")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if is_sidechain {
            continue;
        }

        let timestamp = entry
            .get("timestamp")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        let role = match msg_type {
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "system" => Role::System,
            _ => continue,
        };

        let message_obj = entry.get("message");

        let content = message_obj.and_then(|m| m.get("content"));
        let text = content
            .map(|c| parse::extract_text_from_content(c))
            .unwrap_or_default();

        let model = message_obj
            .and_then(|m| m.get("model"))
            .and_then(|m| m.as_str())
            .map(String::from);

        let mut thinking = None;
        if let Some(serde_json::Value::Array(arr)) = content {
            for item in arr {
                if item.get("type").and_then(|t| t.as_str()) == Some("thinking") {
                    thinking = item
                        .get("thinking")
                        .and_then(|t| t.as_str())
                        .map(String::from);
                    break;
                }
            }
        }

        let (tool_name, tool_input) = extract_tool_use(content);
        let tool_output = extract_tool_result(&entry);

        let clean_text = strip_tool_markers(&text);

        messages.push(Message {
            role,
            timestamp,
            text: clean_text,
            tool_name,
            tool_input,
            tool_output,
            model,
            thinking,
        });
    }

    Ok(messages)
}

fn extract_tool_use(content: Option<&serde_json::Value>) -> (Option<String>, Option<String>) {
    let arr = match content {
        Some(serde_json::Value::Array(arr)) => arr,
        _ => return (None, None),
    };

    for item in arr {
        if item.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
            let name = item
                .get("name")
                .and_then(|n| n.as_str())
                .map(String::from);
            let input = item.get("input").map(|i| i.to_string());
            return (name, input);
        }
    }

    (None, None)
}

fn extract_tool_result(entry: &serde_json::Value) -> Option<String> {
    if let Some(result) = entry.get("toolUseResult") {
        if let Some(s) = result.as_str() {
            return Some(s.to_string());
        }
        if let Some(stdout) = result.get("stdout").and_then(|s| s.as_str()) {
            return Some(stdout.to_string());
        }
        if let Some(content) = result.get("content").and_then(|c| c.as_str()) {
            return Some(content.to_string());
        }
        return Some(result.to_string());
    }
    None
}

fn strip_tool_markers(text: &str) -> String {
    text.lines()
        .filter(|line| !line.starts_with("[tool: ") && !line.starts_with("[result] "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn truncate_string(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}...")
    }
}
