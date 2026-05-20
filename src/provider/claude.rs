use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::model::{Message, Project, Role, SearchResult, Session, SessionMetadata};
use crate::parse;
use crate::provider::Provider;

use crate::search::SearchOptions;

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
                    let parent_id = session.id.clone();
                    sessions.push(session);

                    let subagents_dir = dir.join(&parent_id).join("subagents");
                    if subagents_dir.is_dir() {
                        if let Ok(entries) = std::fs::read_dir(&subagents_dir) {
                            for sa_entry in entries.flatten() {
                                let sa_path = sa_entry.path();
                                if sa_path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                                    if let Some(mut sa_session) = load_session_metadata(&sa_path, &project.name)? {
                                        sa_session.is_subagent = true;
                                        sa_session.parent_session_id = Some(parent_id.clone());
                                        sa_session.id = sa_path.file_stem()
                                            .map(|s| s.to_string_lossy().to_string())
                                            .unwrap_or_else(|| sa_session.id.clone());
                                        let meta_path = sa_path.with_extension("meta.json");
                                        if let Some((agent_type, description)) = load_subagent_meta(&meta_path) {
                                            sa_session.agent_type = Some(agent_type);
                                            sa_session.agent_description = Some(description);
                                        }
                                        sessions.push(sa_session);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        sessions.sort_by(|a, b| b.last_time.cmp(&a.last_time));
        Ok(sessions)
    }

    fn load_messages(&self, session: &Session) -> Result<Vec<Message>> {
        let path = Path::new(&session.file_path);
        load_claude_messages(path, session.is_subagent)
    }

    fn search(&self, opts: &SearchOptions) -> Result<Vec<SearchResult>> {
        let projects = self.scan_projects()?;
        let mut entries = Vec::new();

        for project in &projects {
            let sessions = self.list_sessions(project)?;
            for session in sessions {
                let messages = self.load_messages(&session)?;
                if !messages.is_empty() {
                    entries.push((session, messages, "claude".to_string()));
                }
            }
        }

        Ok(crate::search::rank_and_search(entries, opts))
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

fn load_subagent_meta(path: &Path) -> Option<(String, String)> {
    let data = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&data).ok()?;
    let agent_type = json.get("agentType").and_then(|v| v.as_str())?.to_string();
    let description = json.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    Some((agent_type, description))
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

    let mut files_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut tools_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut has_errors = false;

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

        // Extract metadata from message content
        if let Some(message) = entry.get("message") {
            if let Some(serde_json::Value::Array(content)) = message.get("content") {
                for item in content {
                    let item_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    if item_type == "tool_use" {
                        if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                            tools_set.insert(name.to_string());
                        }
                        if let Some(input) = item.get("input") {
                            extract_file_paths_from_input(input, &mut files_set);
                        }
                    }
                }
            }
        }

        // Detect errors in tool results
        if !has_errors {
            if let Some(result) = entry.get("toolUseResult") {
                let result_text = result.as_str().unwrap_or("");
                if contains_error_pattern(result_text) {
                    has_errors = true;
                }
            }
        }
        if !has_errors && msg_type == "assistant" {
            if let Some(message) = entry.get("message") {
                if let Some(content) = message.get("content") {
                    let text = parse::extract_text_from_content(content);
                    if contains_error_pattern(&text) {
                        has_errors = true;
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
        provider: "claude".to_string(),
        id: session_id,
        file_path: path.to_string_lossy().to_string(),
        project_name: project_name.to_string(),
        message_count,
        first_time,
        last_time,
        summary: final_summary,
        metadata,
        is_subagent: false,
        parent_session_id: None,
        agent_type: None,
        agent_description: None,
    }))
}

fn find_timestamp_fast(line: &[u8]) -> Option<usize> {
    let needle = b"\"timestamp\":\"";
    line.windows(needle.len())
        .position(|w| w == needle)
        .map(|pos| pos + needle.len())
}

fn load_claude_messages(path: &Path, is_subagent: bool) -> Result<Vec<Message>> {
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

        if !is_subagent {
            let is_sidechain = entry
                .get("isSidechain")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if is_sidechain {
                continue;
            }
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

fn extract_file_paths_from_input(
    input: &serde_json::Value,
    files: &mut std::collections::HashSet<String>,
) {
    // Direct file_path field (Read, Write, Edit tools)
    if let Some(fp) = input.get("file_path").and_then(|f| f.as_str()) {
        files.insert(normalize_path(fp));
    }
    // Command field may contain file paths (Bash tool)
    if let Some(cmd) = input.get("command").and_then(|c| c.as_str()) {
        for word in cmd.split_whitespace() {
            if looks_like_file_path(word) {
                files.insert(normalize_path(word));
            }
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
    // Strip home directory prefix for compactness
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
