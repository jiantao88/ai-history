use std::collections::HashMap;

use crate::model::{Message, Role, Session};
use super::{Decision, FileAction, FileChange, SessionDigest};

const DECISION_MARKERS: &[&str] = &[
    "i'll ", "let's ", "instead of ", "rather than ", "better to ",
    "i chose ", "i decided ", "the approach ", "trade-off", "the plan is",
    "i went with", "opting for", "going with",
];

const ISSUE_MARKERS: &[&str] = &[
    "todo", "fixme", "hack", "still need", "doesn't handle",
    "known issue", "limitation", "not yet", "remaining",
    "future work", "needs to be", "should be fixed",
    "doesn't support", "not implemented",
];

const ERROR_PATTERNS: &[&str] = &["error", "failed", "panic", "traceback", "fatal"];

pub fn extract_digest(session: &Session, messages: &[Message]) -> SessionDigest {
    let intent = extract_intent(messages);
    let key_decisions = extract_decisions(messages);
    let code_changes = extract_code_changes(messages);
    let remaining_issues = extract_remaining_issues(messages);
    let conclusion = extract_conclusion(messages);

    let tools_used = session
        .metadata
        .as_ref()
        .map(|m| m.tools_used.clone())
        .unwrap_or_default();
    let had_errors = session
        .metadata
        .as_ref()
        .map(|m| m.has_errors)
        .unwrap_or(false);

    let time_range = if session.first_time == session.last_time {
        session.first_time.clone()
    } else {
        format!("{} — {}", session.first_time, session.last_time)
    };

    let source_mtime = std::fs::metadata(&session.file_path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    SessionDigest {
        session_id: session.id.clone(),
        provider: session.provider.clone(),
        project_name: session.project_name.clone(),
        time_range,
        intent,
        key_decisions,
        code_changes,
        remaining_issues,
        conclusion,
        tools_used,
        had_errors,
        llm_enhanced: false,
        source_mtime,
    }
}

fn extract_intent(messages: &[Message]) -> String {
    for msg in messages {
        if msg.role != Role::User {
            continue;
        }
        if msg.tool_name.is_some() || msg.tool_output.is_some() {
            continue;
        }
        let text = msg.text.trim();
        if text.len() < 10 || is_greeting(text) {
            continue;
        }
        return first_n_sentences(text, 3, 500);
    }
    "Unknown intent".to_string()
}

fn extract_decisions(messages: &[Message]) -> Vec<Decision> {
    let mut decisions: Vec<Decision> = Vec::new();

    for msg in messages {
        if msg.role != Role::Assistant {
            continue;
        }

        // Extract from thinking blocks
        if let Some(thinking) = &msg.thinking {
            extract_decisions_from_text(thinking, &mut decisions);
        }

        // Extract from assistant text (skip tool call messages)
        if msg.tool_name.is_none() {
            let text = msg.text.trim();
            if !text.is_empty() {
                extract_decisions_from_text(text, &mut decisions);
            }
        }
    }

    dedup_decisions(&mut decisions);
    decisions.truncate(5);
    decisions
}

fn extract_decisions_from_text(text: &str, decisions: &mut Vec<Decision>) {
    let sentences = split_sentences(text);
    for (i, sentence) in sentences.iter().enumerate() {
        let lower = sentence.to_lowercase();
        if DECISION_MARKERS.iter().any(|m| lower.contains(m)) {
            let rationale = sentences.get(i + 1).and_then(|next| {
                let next_lower = next.to_lowercase();
                if next_lower.starts_with("because")
                    || next_lower.starts_with("since")
                    || next_lower.starts_with("this way")
                    || next_lower.starts_with("this ensures")
                {
                    Some(next.trim().to_string())
                } else {
                    None
                }
            });
            decisions.push(Decision {
                description: sentence.trim().to_string(),
                rationale,
            });
        }
    }
}

fn extract_code_changes(messages: &[Message]) -> Vec<FileChange> {
    let mut file_actions: HashMap<String, (FileAction, Vec<String>, usize)> = HashMap::new();

    for (i, msg) in messages.iter().enumerate() {
        let tool = match &msg.tool_name {
            Some(t) => t.as_str(),
            None => continue,
        };

        match tool {
            "Write" | "write_file" => {
                if let Some(path) = parse_file_path(&msg.tool_input) {
                    let desc = find_preceding_assistant_text(messages, i);
                    let entry = file_actions.entry(path).or_insert((FileAction::Created, Vec::new(), 0));
                    if let Some(d) = desc {
                        entry.1.push(d);
                    }
                    entry.2 += 1;
                }
            }
            "Edit" | "edit_file" | "MultiEdit" | "multi_edit_file" => {
                if let Some(path) = parse_file_path(&msg.tool_input) {
                    let desc = find_preceding_assistant_text(messages, i);
                    let entry = file_actions.entry(path).or_insert((FileAction::Modified, Vec::new(), 0));
                    entry.0 = FileAction::Modified;
                    if let Some(d) = desc {
                        entry.1.push(d);
                    }
                    entry.2 += 1;
                }
            }
            "Bash" | "shell" | "exec_command" => {
                if let Some(input) = &msg.tool_input {
                    let cmd = extract_command(input);
                    if cmd.contains("rm ") || cmd.contains("rm\t") {
                        for word in cmd.split_whitespace().skip(1) {
                            if looks_like_file_path(word) {
                                let path = normalize_path(word);
                                file_actions.insert(path, (FileAction::Deleted, Vec::new(), 1));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let mut changes: Vec<FileChange> = file_actions
        .into_iter()
        .map(|(path, (action, descs, count))| {
            let description = if descs.is_empty() {
                if count > 1 {
                    Some(format!("({count} changes)"))
                } else {
                    None
                }
            } else {
                let first = descs.into_iter().next().unwrap();
                if count > 1 {
                    Some(format!("{first} ({count} changes)"))
                } else {
                    Some(first)
                }
            };
            FileChange {
                file_path: path,
                action,
                description,
            }
        })
        .collect();

    changes.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    changes
}

fn extract_remaining_issues(messages: &[Message]) -> Vec<String> {
    let mut issues: Vec<String> = Vec::new();

    // Check last few assistant messages for issue markers
    let assistant_msgs: Vec<&Message> = messages
        .iter()
        .rev()
        .filter(|m| m.role == Role::Assistant && m.tool_name.is_none())
        .take(3)
        .collect();

    for msg in &assistant_msgs {
        let sentences = split_sentences(&msg.text);
        for sentence in &sentences {
            let lower = sentence.to_lowercase();
            if ISSUE_MARKERS.iter().any(|m| lower.contains(m)) {
                let trimmed = sentence.trim().to_string();
                if !trimmed.is_empty() && !issues.iter().any(|existing| existing.contains(&trimmed)) {
                    issues.push(trimmed);
                }
            }
        }
    }

    // Check for unresolved errors in tool outputs
    let mut error_files: Vec<(usize, String)> = Vec::new();
    for (i, msg) in messages.iter().enumerate() {
        if let Some(output) = &msg.tool_output {
            if contains_error_pattern(output) {
                if let Some(path) = parse_file_path(&msg.tool_input) {
                    error_files.push((i, path));
                }
            }
        }
    }

    for (err_idx, err_file) in &error_files {
        let resolved = messages.iter().skip(err_idx + 1).any(|m| {
            matches!(m.tool_name.as_deref(), Some("Edit" | "Write" | "edit_file" | "write_file"))
                && parse_file_path(&m.tool_input).as_deref() == Some(err_file.as_str())
        });
        if !resolved {
            let issue = format!("Unresolved error in {err_file}");
            if !issues.contains(&issue) {
                issues.push(issue);
            }
        }
    }

    issues.truncate(5);
    issues
}

fn extract_conclusion(messages: &[Message]) -> String {
    for msg in messages.iter().rev() {
        if msg.role != Role::Assistant {
            continue;
        }
        if msg.tool_name.is_some() {
            continue;
        }
        let text = msg.text.trim();
        if text.is_empty() {
            continue;
        }
        return first_n_sentences(text, 3, 500);
    }
    "No conclusion found".to_string()
}

// --- Utility functions ---

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '.' | '?' | '!') {
            let trimmed = current.trim().to_string();
            if trimmed.len() > 5 {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() && trimmed.len() > 5 {
        sentences.push(trimmed);
    }
    sentences
}

fn first_n_sentences(text: &str, n: usize, max_chars: usize) -> String {
    let sentences = split_sentences(text);
    let mut result = String::new();
    for (i, s) in sentences.iter().take(n).enumerate() {
        if result.len() + s.len() > max_chars {
            break;
        }
        if i > 0 {
            result.push(' ');
        }
        result.push_str(s);
    }
    if result.is_empty() {
        let boundary = text.char_indices()
            .take_while(|(i, _)| *i < max_chars)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(text.len().min(max_chars));
        text[..boundary].trim().to_string()
    } else {
        result
    }
}

fn is_greeting(text: &str) -> bool {
    let lower = text.to_lowercase();
    let trimmed = lower.trim();
    matches!(
        trimmed,
        "hi" | "hello" | "hey" | "你好" | "嗨" | "ok" | "okay" | "yes" | "no"
            | "thanks" | "thank you" | "谢谢" | "好的" | "继续" | "continue"
    )
}

fn parse_file_path(tool_input: &Option<String>) -> Option<String> {
    let input = tool_input.as_ref()?;
    let val: serde_json::Value = serde_json::from_str(input).ok()?;
    if let Some(fp) = val.get("file_path").and_then(|f| f.as_str()) {
        return Some(normalize_path(fp));
    }
    if let Some(fp) = val.get("filePath").and_then(|f| f.as_str()) {
        return Some(normalize_path(fp));
    }
    None
}

fn extract_command(tool_input: &str) -> String {
    serde_json::from_str::<serde_json::Value>(tool_input)
        .ok()
        .and_then(|v| v.get("command").and_then(|c| c.as_str()).map(String::from))
        .unwrap_or_default()
}

fn find_preceding_assistant_text(messages: &[Message], tool_idx: usize) -> Option<String> {
    for i in (0..tool_idx).rev() {
        let msg = &messages[i];
        if msg.role == Role::User {
            break;
        }
        if msg.role == Role::Assistant && msg.tool_name.is_none() {
            let text = msg.text.trim();
            if !text.is_empty() {
                return Some(first_n_sentences(text, 1, 200));
            }
        }
    }
    None
}

fn looks_like_file_path(s: &str) -> bool {
    if s.len() < 3 || s.starts_with('-') {
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

fn contains_error_pattern(text: &str) -> bool {
    let lower = text.to_lowercase();
    ERROR_PATTERNS.iter().any(|p| lower.contains(p))
}

fn dedup_decisions(decisions: &mut Vec<Decision>) {
    let mut i = 0;
    while i < decisions.len() {
        let mut j = i + 1;
        while j < decisions.len() {
            let a = &decisions[i].description.to_lowercase();
            let b = &decisions[j].description.to_lowercase();
            if a.contains(b.as_str()) || b.contains(a.as_str()) {
                decisions.remove(j);
            } else {
                j += 1;
            }
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Role, Session, SessionMetadata};

    fn msg(role: Role, text: &str) -> Message {
        Message {
            role,
            timestamp: "2026-05-08T14:00:00Z".to_string(),
            text: text.to_string(),
            tool_name: None,
            tool_input: None,
            tool_output: None,
            model: None,
            thinking: None,
        }
    }

    fn tool_msg(tool: &str, input: &str, output: &str) -> Message {
        Message {
            role: Role::Assistant,
            timestamp: "2026-05-08T14:05:00Z".to_string(),
            text: String::new(),
            tool_name: Some(tool.to_string()),
            tool_input: Some(input.to_string()),
            tool_output: Some(output.to_string()),
            model: None,
            thinking: None,
        }
    }

    fn test_session() -> Session {
        Session {
            provider: "claude".to_string(),
            id: "test-session-123".to_string(),
            file_path: "/tmp/test.jsonl".to_string(),
            project_name: "test-project".to_string(),
            message_count: 10,
            first_time: "2026-05-08T14:00:00Z".to_string(),
            last_time: "2026-05-08T16:00:00Z".to_string(),
            summary: None,
            metadata: Some(SessionMetadata {
                files_touched: vec!["src/main.rs".to_string()],
                tools_used: vec!["Edit".to_string(), "Bash".to_string()],
                has_errors: false,
                languages: vec!["Rust".to_string()],
            }),
            is_subagent: false,
            parent_session_id: None,
            agent_type: None,
            agent_description: None,
        }
    }

    #[test]
    fn test_extract_intent() {
        let messages = vec![
            msg(Role::User, "Fix the authentication bug in the login flow. Users are getting 401 errors when their session expires."),
            msg(Role::Assistant, "I'll look into the authentication issue."),
        ];
        let intent = extract_intent(&messages);
        assert!(intent.contains("authentication bug"));
        assert!(intent.contains("401"));
    }

    #[test]
    fn test_extract_intent_skips_greeting() {
        let messages = vec![
            msg(Role::User, "hi"),
            msg(Role::User, "Can you add dark mode support to the settings page?"),
            msg(Role::Assistant, "Sure, I'll add dark mode."),
        ];
        let intent = extract_intent(&messages);
        assert!(intent.contains("dark mode"));
    }

    #[test]
    fn test_extract_decisions() {
        let mut thinking_msg = msg(Role::Assistant, "I'll implement this using a middleware approach.");
        thinking_msg.thinking = Some(
            "I'll use a middleware pattern instead of modifying each route handler. \
             Because this keeps the auth logic centralized and easier to maintain."
                .to_string(),
        );
        let messages = vec![
            msg(Role::User, "Fix the auth issue"),
            thinking_msg,
        ];
        let decisions = extract_decisions(&messages);
        assert!(!decisions.is_empty());
        assert!(decisions[0].description.contains("middleware"));
    }

    #[test]
    fn test_extract_code_changes() {
        let messages = vec![
            msg(Role::User, "Add a new config file"),
            msg(Role::Assistant, "I'll create the configuration file."),
            tool_msg(
                "Write",
                r#"{"file_path": "/Users/test/project/src/config.rs"}"#,
                "File written successfully",
            ),
            msg(Role::Assistant, "Now I'll update the module declaration."),
            tool_msg(
                "Edit",
                r#"{"file_path": "/Users/test/project/src/main.rs"}"#,
                "Edit applied",
            ),
        ];
        let changes = extract_code_changes(&messages);
        assert_eq!(changes.len(), 2);
        let config = changes.iter().find(|c| c.file_path.contains("config")).unwrap();
        assert!(matches!(config.action, FileAction::Created));
        let main = changes.iter().find(|c| c.file_path.contains("main")).unwrap();
        assert!(matches!(main.action, FileAction::Modified));
    }

    #[test]
    fn test_extract_remaining_issues() {
        let messages = vec![
            msg(Role::User, "Fix the bug"),
            msg(Role::Assistant, "I've fixed the main issue. There's still a known issue with edge cases where the input is empty. TODO: add validation for empty strings."),
        ];
        let issues = extract_remaining_issues(&messages);
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_extract_conclusion() {
        let messages = vec![
            msg(Role::User, "Add the feature"),
            msg(Role::Assistant, "Working on it."),
            tool_msg("Edit", r#"{"file_path": "/tmp/test.rs"}"#, "ok"),
            msg(Role::Assistant, "Done. I've added the new endpoint with proper error handling and tests. The API now supports pagination."),
        ];
        let conclusion = extract_conclusion(&messages);
        assert!(conclusion.contains("endpoint"));
        assert!(conclusion.contains("pagination"));
    }

    #[test]
    fn test_full_digest() {
        let mut thinking_msg = msg(Role::Assistant, "I'll use reqwest for HTTP calls.");
        thinking_msg.thinking = Some("I'll use reqwest instead of hyper. Because reqwest has a simpler API for our use case.".to_string());

        let messages = vec![
            msg(Role::User, "Add an HTTP client to fetch remote configs from the API server."),
            thinking_msg,
            msg(Role::Assistant, "Let me create the HTTP client module."),
            tool_msg("Write", r#"{"file_path": "/Users/test/src/http.rs"}"#, "ok"),
            msg(Role::Assistant, "The HTTP client is ready. It supports GET and POST requests with automatic retry. There's a known limitation: it doesn't handle streaming responses yet."),
        ];

        let session = test_session();
        let digest = extract_digest(&session, &messages);

        assert!(digest.intent.contains("HTTP client"));
        assert!(!digest.key_decisions.is_empty());
        assert!(!digest.code_changes.is_empty());
        assert!(!digest.remaining_issues.is_empty());
        assert!(digest.conclusion.contains("retry"));
    }

    #[test]
    fn test_split_sentences() {
        let text = "First sentence. Second sentence. Third one here.";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 3);
        assert!(sentences[0].contains("First"));
        assert!(sentences[2].contains("Third"));
    }
}
