use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::model::{Message, Project, Role, SearchResult, Session, SessionMetadata};
use crate::parse;
use crate::provider::Provider;
use crate::search::SearchOptions;

pub struct CodexProvider {
    base_path: Option<PathBuf>,
}

impl CodexProvider {
    pub fn new() -> Self {
        let base = get_base_path();
        Self { base_path: base }
    }
}

fn get_base_path() -> Option<PathBuf> {
    if let Ok(codex_home) = std::env::var("CODEX_HOME") {
        let path = PathBuf::from(&codex_home);
        if path.exists() {
            return Some(path);
        }
    }
    dirs::home_dir().map(|h| h.join(".codex")).filter(|p| p.exists())
}

impl Provider for CodexProvider {
    fn id(&self) -> &str {
        "codex"
    }

    fn display_name(&self) -> &str {
        "Codex CLI"
    }

    fn is_available(&self) -> bool {
        let Some(ref base) = self.base_path else {
            return false;
        };
        let sessions = base.join("sessions");
        let archived = base.join("archived_sessions");
        (sessions.exists() && sessions.is_dir())
            || (archived.exists() && archived.is_dir())
    }

    fn scan_projects(&self) -> Result<Vec<Project>> {
        let Some(ref base) = self.base_path else {
            return Ok(Vec::new());
        };

        let mut cwd_map: std::collections::HashMap<String, ProjectAcc> =
            std::collections::HashMap::new();

        for dir_name in &["sessions", "archived_sessions"] {
            let dir = base.join(dir_name);
            if !dir.exists() {
                continue;
            }
            for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if !is_rollout_file(path) {
                    continue;
                }
                if let Some((cwd, mtime)) = extract_codex_session_info(path) {
                    let acc = cwd_map.entry(cwd.clone()).or_insert_with(|| ProjectAcc {
                        cwd,
                        session_count: 0,
                        last_modified: String::new(),
                    });
                    acc.session_count += 1;
                    if mtime > acc.last_modified {
                        acc.last_modified = mtime;
                    }
                }
            }
        }

        let mut projects: Vec<Project> = cwd_map
            .into_values()
            .map(|acc| Project {
                provider: "codex".to_string(),
                name: acc.cwd.clone(),
                path: acc.cwd,
                session_count: acc.session_count,
                last_modified: acc.last_modified,
            })
            .collect();

        projects.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        Ok(projects)
    }

    fn list_sessions(&self, project: &Project) -> Result<Vec<Session>> {
        let Some(ref base) = self.base_path else {
            return Ok(Vec::new());
        };

        let mut sessions = Vec::new();

        for dir_name in &["sessions", "archived_sessions"] {
            let dir = base.join(dir_name);
            if !dir.exists() {
                continue;
            }
            for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if !is_rollout_file(path) {
                    continue;
                }
                if let Some(session) = load_codex_session_metadata(path, &project.name)? {
                    if session.project_name == project.name {
                        sessions.push(session);
                    }
                }
            }
        }

        sessions.sort_by(|a, b| b.last_time.cmp(&a.last_time));
        Ok(sessions)
    }

    fn load_messages(&self, session: &Session) -> Result<Vec<Message>> {
        let path = Path::new(&session.file_path);
        load_codex_messages(path)
    }

    fn search(&self, opts: &SearchOptions) -> Result<Vec<SearchResult>> {
        let projects = self.scan_projects()?;
        let mut entries = Vec::new();

        for project in &projects {
            let sessions = self.list_sessions(&project)?;
            for session in sessions {
                let messages = self.load_messages(&session)?;
                if !messages.is_empty() {
                    entries.push((session, messages, "codex".to_string()));
                }
            }
        }

        Ok(crate::search::rank_and_search(entries, opts))
    }
}

struct ProjectAcc {
    cwd: String,
    session_count: usize,
    last_modified: String,
}

fn is_rollout_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name.starts_with("rollout-") && name.ends_with(".jsonl")
}

fn extract_codex_session_info(path: &Path) -> Option<(String, String)> {
    let mmap = parse::mmap_file(path).ok()?;
    let ranges = parse::find_line_ranges(&mmap);

    let mut cwd = None;
    let mut last_ts = String::new();

    for &(start, end) in ranges.iter().take(20) {
        let line = &mmap[start..end];
        let entry = parse::parse_jsonl_line(line)?;

        let line_type = entry.get("type").and_then(|t| t.as_str()).unwrap_or("");

        if line_type == "session_meta" {
            if let Some(payload) = entry.get("payload") {
                cwd = payload
                    .get("cwd")
                    .and_then(|c| c.as_str())
                    .map(String::from);
            }
        }

        if let Some(ts) = entry.get("ts").and_then(|t| t.as_str()) {
            if ts > last_ts.as_str() {
                last_ts = ts.to_string();
            }
        }

        if cwd.is_some() {
            break;
        }
    }

    if last_ts.is_empty() {
        let mtime = path.metadata().ok()?.modified().ok()?;
        let dt: chrono::DateTime<chrono::Utc> = mtime.into();
        last_ts = dt.to_rfc3339();
    }

    cwd.map(|c| (c, last_ts))
}

fn load_codex_session_metadata(path: &Path, target_cwd: &str) -> Result<Option<Session>> {
    let mmap = parse::mmap_file(path)?;
    let ranges = parse::find_line_ranges(&mmap);

    let mut session_id = String::new();
    let mut cwd = String::new();
    let mut first_time = String::new();
    let mut last_time = String::new();
    let mut message_count = 0usize;
    let mut first_user_text: Option<String> = None;
    let mut model: Option<String> = None;

    let mut files_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut tools_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut has_errors = false;

    for &(start, end) in &ranges {
        let line = &mmap[start..end];
        let Some(entry) = parse::parse_jsonl_line(line) else {
            continue;
        };

        let line_type = entry.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let ts = entry
            .get("ts")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        if !ts.is_empty() {
            if first_time.is_empty() {
                first_time = ts.clone();
            }
            last_time = ts;
        }

        match line_type {
            "session_meta" => {
                if let Some(payload) = entry.get("payload") {
                    session_id = payload
                        .get("id")
                        .and_then(|i| i.as_str())
                        .unwrap_or("")
                        .to_string();
                    cwd = payload
                        .get("cwd")
                        .and_then(|c| c.as_str())
                        .unwrap_or("")
                        .to_string();
                }
            }
            "turn_context" if model.is_none() => {
                if let Some(payload) = entry.get("payload") {
                    model = payload
                        .get("model")
                        .and_then(|m| m.as_str())
                        .map(String::from);
                }
            }
            "response_item" => {
                if let Some(payload) = entry.get("payload") {
                    let item_type = payload.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    if item_type == "message" {
                        let role_str = payload
                            .get("role")
                            .and_then(|r| r.as_str())
                            .unwrap_or("");
                        if role_str == "user" || role_str == "assistant" {
                            message_count += 1;
                        }
                        if role_str == "user" && first_user_text.is_none() {
                            if let Some(content) = payload.get("content") {
                                let text = extract_codex_text(content);
                                if !text.is_empty() {
                                    first_user_text =
                                        Some(truncate_string(&text, 100));
                                }
                            }
                        }
                    }
                    if item_type == "function_call" {
                        if let Some(name) = payload.get("name").and_then(|n| n.as_str()) {
                            tools_set.insert(map_tool_name(name).to_string());
                        }
                        if let Some(args) = payload.get("arguments").and_then(|a| a.as_str()) {
                            extract_file_paths_from_args(args, &mut files_set);
                        }
                    }
                    if item_type == "function_call_output" && !has_errors {
                        if let Some(output) = payload.get("output").and_then(|o| o.as_str()) {
                            if contains_error_pattern(output) {
                                has_errors = true;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if cwd != target_cwd {
        return Ok(None);
    }

    if session_id.is_empty() {
        session_id = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
    }

    let mut files_touched: Vec<String> = files_set.into_iter().collect();
    files_touched.sort();
    let mut tools_used: Vec<String> = tools_set.into_iter().collect();
    tools_used.sort();
    let languages = infer_languages(&files_touched);

    let metadata = if files_touched.is_empty() && tools_used.is_empty() && !has_errors {
        None
    } else {
        Some(SessionMetadata {
            files_touched,
            tools_used,
            has_errors,
            languages,
        })
    };

    Ok(Some(Session {
        provider: "codex".to_string(),
        id: session_id,
        file_path: path.to_string_lossy().to_string(),
        project_name: cwd,
        message_count,
        first_time,
        last_time,
        summary: first_user_text,
        metadata,
    }))
}

fn load_codex_messages(path: &Path) -> Result<Vec<Message>> {
    let mmap = parse::mmap_file(path).context("Failed to open Codex session file")?;
    let ranges = parse::find_line_ranges(&mmap);
    let mut messages = Vec::new();

    for &(start, end) in &ranges {
        let line = &mmap[start..end];
        let Some(entry) = parse::parse_jsonl_line(line) else {
            continue;
        };

        let line_type = entry.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let ts = entry
            .get("ts")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        match line_type {
            "response_item" => {
                if let Some(payload) = entry.get("payload") {
                    if let Some(msg) = convert_response_item(payload, &ts) {
                        messages.push(msg);
                    }
                }
            }
            "event_msg" => {
                if let Some(payload) = entry.get("payload") {
                    let event_type = payload.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    if event_type == "user_message" || event_type == "agent_message" {
                        continue;
                    }
                    if let Some(msg) = convert_event_msg(payload, &ts) {
                        messages.push(msg);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(messages)
}

fn convert_response_item(payload: &serde_json::Value, ts: &str) -> Option<Message> {
    let item_type = payload.get("type").and_then(|t| t.as_str())?;

    match item_type {
        "message" => {
            let role_str = payload.get("role").and_then(|r| r.as_str()).unwrap_or("");
            let role = match role_str {
                "user" => Role::User,
                "assistant" => Role::Assistant,
                "system" => Role::System,
                _ => return None,
            };

            let text = payload
                .get("content")
                .map(|c| extract_codex_text(c))
                .unwrap_or_default();

            let model = payload
                .get("model")
                .and_then(|m| m.as_str())
                .map(String::from);

            let thinking = extract_codex_thinking(payload.get("content"));

            Some(Message {
                role,
                timestamp: ts.to_string(),
                text,
                tool_name: None,
                tool_input: None,
                tool_output: None,
                model,
                thinking,
            })
        }
        "function_call" => {
            let name = payload
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("unknown");
            let mapped_name = map_tool_name(name);
            let arguments = payload
                .get("arguments")
                .and_then(|a| a.as_str())
                .map(String::from);

            Some(Message {
                role: Role::Tool,
                timestamp: ts.to_string(),
                text: String::new(),
                tool_name: Some(mapped_name.to_string()),
                tool_input: arguments,
                tool_output: None,
                model: None,
                thinking: None,
            })
        }
        "function_call_output" => {
            let output = payload
                .get("output")
                .and_then(|o| o.as_str())
                .map(String::from);

            Some(Message {
                role: Role::Tool,
                timestamp: ts.to_string(),
                text: String::new(),
                tool_name: Some("result".to_string()),
                tool_input: None,
                tool_output: output,
                model: None,
                thinking: None,
            })
        }
        _ => None,
    }
}

fn convert_event_msg(payload: &serde_json::Value, ts: &str) -> Option<Message> {
    let event_type = payload.get("type").and_then(|t| t.as_str())?;

    match event_type {
        "context_compacted" | "task_started" | "turn_aborted" => Some(Message {
            role: Role::System,
            timestamp: ts.to_string(),
            text: format!("[{event_type}]"),
            tool_name: None,
            tool_input: None,
            tool_output: None,
            model: None,
            thinking: None,
        }),
        _ => None,
    }
}

fn extract_codex_text(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            let mut parts = Vec::new();
            for item in arr {
                let t = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
                match t {
                    "input_text" | "output_text" | "text" => {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            parts.push(text.to_string());
                        }
                    }
                    "refusal" => {
                        if let Some(text) = item.get("refusal").and_then(|t| t.as_str()) {
                            parts.push(format!("[Refusal] {text}"));
                        }
                    }
                    _ => {}
                }
            }
            parts.join("\n")
        }
        _ => String::new(),
    }
}

fn extract_codex_thinking(content: Option<&serde_json::Value>) -> Option<String> {
    let arr = content?.as_array()?;
    for item in arr {
        let t = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if t == "reasoning" || t == "thinking" {
            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                return Some(text.to_string());
            }
        }
    }
    None
}

fn map_tool_name(name: &str) -> &str {
    match name {
        "exec_command" | "shell" | "write_stdin" => "Bash",
        _ => name,
    }
}

fn truncate_string(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}...")
    }
}

fn extract_file_paths_from_args(
    args_str: &str,
    files: &mut std::collections::HashSet<String>,
) {
    if let Ok(args) = serde_json::from_str::<serde_json::Value>(args_str) {
        if let Some(cmd) = args.get("command").and_then(|c| c.as_str()) {
            for word in cmd.split_whitespace() {
                if looks_like_file_path(word) {
                    files.insert(normalize_path(word));
                }
            }
        }
        if let Some(fp) = args.get("file_path").and_then(|f| f.as_str()) {
            files.insert(normalize_path(fp));
        }
    }
}

fn looks_like_file_path(s: &str) -> bool {
    if s.len() < 3 {
        return false;
    }
    let has_ext = s.rfind('.').map(|i| i > 0 && i < s.len() - 1).unwrap_or(false);
    let has_sep = s.contains('/');
    (has_ext && has_sep) || s.starts_with("./") || s.starts_with("src/") || s.starts_with("tests/")
}

fn normalize_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if let Some(stripped) = path.strip_prefix(home_str.as_ref()) {
            return format!("~{stripped}");
        }
    }
    path.to_string()
}

const ERROR_PATTERNS: &[&str] = &["error", "failed", "panic", "traceback", "fatal"];

fn contains_error_pattern(text: &str) -> bool {
    let lower = text.to_lowercase();
    ERROR_PATTERNS.iter().any(|p| lower.contains(p))
}

fn infer_languages(files: &[String]) -> Vec<String> {
    let mut langs: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for f in files {
        if let Some(ext) = std::path::Path::new(f).extension().and_then(|e| e.to_str()) {
            if let Some(lang) = ext_to_language(ext) {
                langs.insert(lang);
            }
        }
    }
    let mut result: Vec<String> = langs.into_iter().map(String::from).collect();
    result.sort();
    result
}

fn ext_to_language(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some("Rust"),
        "ts" | "tsx" => Some("TypeScript"),
        "js" | "jsx" => Some("JavaScript"),
        "py" => Some("Python"),
        "go" => Some("Go"),
        "java" => Some("Java"),
        "kt" | "kts" => Some("Kotlin"),
        "swift" => Some("Swift"),
        "rb" => Some("Ruby"),
        "c" | "h" => Some("C"),
        "cpp" | "cc" | "cxx" | "hpp" => Some("C++"),
        "cs" => Some("C#"),
        "html" | "htm" => Some("HTML"),
        "css" | "scss" | "sass" => Some("CSS"),
        "sql" => Some("SQL"),
        "sh" | "bash" | "zsh" => Some("Shell"),
        "toml" | "yaml" | "yml" | "json" => Some("Config"),
        "md" => Some("Markdown"),
        _ => None,
    }
}
