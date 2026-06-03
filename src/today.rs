use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::model::{Message, Role, Session};
use crate::provider::ProviderRegistry;

#[derive(Debug)]
pub struct TodayOptions<'a> {
    pub project: Option<&'a str>,
    pub date: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TodayReport {
    pub project: String,
    pub date: String,
    pub entries: Vec<WorklogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorklogEntry {
    pub title: String,
    pub provider: String,
    pub session_id: String,
    pub session_id_full: String,
    pub project: String,
    pub first_time: Option<String>,
    pub last_time: Option<String>,
    pub message_count: usize,
    pub files_touched: Vec<String>,
    pub summary: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorklogTitle {
    pub title: String,
    pub provider: String,
    pub session_id: String,
}

#[derive(Debug, Clone)]
struct ProjectFilter {
    raw: String,
    canonical: Option<String>,
}

pub fn build_today_report(
    registry: &ProviderRegistry,
    opts: &TodayOptions<'_>,
    provider_filter: Option<&[String]>,
) -> Result<TodayReport> {
    let date = parse_target_date(opts.date)?;
    let project_filter = resolve_project_filter(opts.project)?;
    let mut entries = Vec::new();

    let sessions = registry.list_all_sessions(provider_filter)?;
    for session in sessions {
        if session.is_subagent || session.message_count <= 1 {
            continue;
        }
        if !project_matches(&session, &project_filter) {
            continue;
        }
        if !session_overlaps_date(&session, date) {
            continue;
        }

        let messages = registry
            .get(&session.provider)
            .and_then(|provider| provider.load_messages(&session).ok())
            .unwrap_or_default();

        entries.push(entry_from_session(&session, &messages));
    }

    let mut entries = dedupe_entries(entries);
    entries.sort_by_key(sort_key);

    Ok(TodayReport {
        project: display_project(&project_filter.raw),
        date: date.format("%Y-%m-%d").to_string(),
        entries,
    })
}

pub fn title_entries(report: &TodayReport) -> Vec<WorklogTitle> {
    report
        .entries
        .iter()
        .map(|entry| WorklogTitle {
            title: entry.title.clone(),
            provider: entry.provider.clone(),
            session_id: entry.session_id.clone(),
        })
        .collect()
}

fn parse_target_date(date: Option<&str>) -> Result<NaiveDate> {
    if let Some(date) = date {
        return NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid date: {date}. Use YYYY-MM-DD"));
    }
    Ok(Local::now().date_naive())
}

fn resolve_project_filter(project: Option<&str>) -> Result<ProjectFilter> {
    let raw = match project {
        Some(p) if !p.trim().is_empty() => p.trim().to_string(),
        _ => ".".to_string(),
    };

    let canonical = if raw == "." {
        Some(canonical_path(std::env::current_dir()?)?)
    } else {
        let path = PathBuf::from(&raw);
        if path.is_absolute() || raw.contains('/') {
            std::fs::canonicalize(&path)
                .ok()
                .map(canonical_path)
                .transpose()?
        } else {
            None
        }
    };

    Ok(ProjectFilter { raw, canonical })
}

fn canonical_path(path: PathBuf) -> Result<String> {
    Ok(path.to_string_lossy().to_string())
}

fn project_matches(session: &Session, filter: &ProjectFilter) -> bool {
    if let Some(canonical) = &filter.canonical {
        return session.project_name == *canonical
            || session.file_path.contains(canonical)
            || path_equivalent(&session.project_name, canonical);
    }

    let needle = filter.raw.as_str();
    session.project_name.contains(needle) || session.file_path.contains(needle)
}

fn path_equivalent(left: &str, right: &str) -> bool {
    let left_path = Path::new(left);
    let right_path = Path::new(right);
    left_path == right_path
}

fn display_project(project: &str) -> String {
    let path = if project == "." {
        std::env::current_dir()
            .ok()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| project.to_string())
    } else {
        project.to_string()
    };

    if let Some(home) = dirs::home_dir() {
        let home = home.to_string_lossy().to_string();
        if let Some(rest) = path.strip_prefix(&home) {
            return format!("~{rest}");
        }
    }
    path
}

fn session_overlaps_date(session: &Session, date: NaiveDate) -> bool {
    let start = parse_local_datetime(&session.first_time);
    let end = parse_local_datetime(&session.last_time).or(start);

    match (start, end) {
        (Some(start), Some(end)) => {
            let start_date = start.date_naive();
            let end_date = end.date_naive();
            start_date <= date && date <= end_date
        }
        (Some(start), None) => start.date_naive() == date,
        (None, Some(end)) => end.date_naive() == date,
        (None, None) => false,
    }
}

fn parse_local_datetime(ts: &str) -> Option<DateTime<Local>> {
    chrono::DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|dt| dt.with_timezone(&Local))
}

fn entry_from_session(session: &Session, messages: &[Message]) -> WorklogEntry {
    let files_touched = session
        .metadata
        .as_ref()
        .map(|m| m.files_touched.clone())
        .unwrap_or_default();

    let title = extract_title(session, messages, &files_touched);
    let summary = extract_summary(session, messages, &title, &files_touched);

    WorklogEntry {
        title,
        provider: session.provider.clone(),
        session_id: short_id(&session.id),
        session_id_full: session.id.clone(),
        project: session.project_name.clone(),
        first_time: normalize_time(&session.first_time),
        last_time: normalize_time(&session.last_time),
        message_count: session.message_count,
        files_touched,
        summary,
    }
}

fn extract_title(session: &Session, messages: &[Message], files_touched: &[String]) -> String {
    if let Some(summary) = session.summary.as_deref() {
        if let Some(title) = clean_title_candidate(summary) {
            return title;
        }
    }

    for message in messages {
        if !matches!(message.role, Role::User) {
            continue;
        }
        if let Some(title) = clean_title_candidate(&message.text) {
            return title;
        }
    }

    if let Some(file) = files_touched.first() {
        return truncate_chars(&format!("修改 {}", compact_file_name(file)), 60);
    }

    format!("未命名会话：{}/{}", session.provider, short_id(&session.id))
}

fn extract_summary(
    session: &Session,
    messages: &[Message],
    title: &str,
    files_touched: &[String],
) -> Vec<String> {
    let mut bullets = Vec::new();
    bullets.push(title.to_string());

    if !files_touched.is_empty() {
        let files = files_touched
            .iter()
            .take(3)
            .map(|f| compact_file_name(f))
            .collect::<Vec<_>>()
            .join(", ");
        bullets.push(format!("涉及文件：{files}"));
    }

    if let Some(metadata) = &session.metadata {
        if !metadata.tools_used.is_empty() {
            let tools = metadata
                .tools_used
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            bullets.push(format!("使用工具：{tools}"));
        }
        if metadata.has_errors {
            bullets.push("包含错误排查或失败输出".to_string());
        }
    }

    if bullets.len() == 1 {
        if let Some(extra) = messages
            .iter()
            .filter(|m| matches!(m.role, Role::User | Role::Assistant))
            .filter_map(|m| clean_title_candidate(&m.text))
            .find(|line| line != title)
        {
            bullets.push(extra);
        }
    }

    bullets.truncate(5);
    bullets
}

fn clean_title_candidate(text: &str) -> Option<String> {
    let text = remove_prompt_blocks(text);

    if let Some(args) = extract_between_tags(&text, "command-args") {
        if let Some(title) = clean_title_line(args) {
            return Some(title);
        }
    }

    if let Some(message) = extract_between_tags(&text, "command-message") {
        if let Some(title) = clean_title_line(message) {
            return Some(title);
        }
    }

    text.lines().find_map(clean_title_line)
}

fn clean_title_line(line: &str) -> Option<String> {
    let cleaned = clean_text(line);
    if cleaned.is_empty() || is_bad_title(&cleaned) {
        return None;
    }
    Some(truncate_chars(&cleaned, 60))
}

fn clean_text(text: &str) -> String {
    let no_images = remove_markdown_images(text)
        .replace("[Image #1]", "")
        .replace("[Image #2]", "")
        .replace("[Image #3]", "");
    let no_links = replace_markdown_links(&no_images);
    let no_tags = strip_xml_tags(&no_links);
    let no_fences = strip_code_fence_markers(&no_tags);
    no_fences.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn remove_markdown_images(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("![") {
        result.push_str(&rest[..start]);
        let Some(end) = rest[start..].find(')') else {
            rest = &rest[start + 2..];
            continue;
        };
        rest = &rest[start + end + 1..];
    }
    result.push_str(rest);
    result
}

fn replace_markdown_links(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(start) = rest.find('[') {
        result.push_str(&rest[..start]);
        let Some(close_bracket) = rest[start..].find(']') else {
            result.push_str(&rest[start..]);
            return result;
        };
        let after_bracket = start + close_bracket + 1;
        if !rest[after_bracket..].starts_with('(') {
            result.push('[');
            rest = &rest[start + 1..];
            continue;
        }
        let Some(close_paren) = rest[after_bracket..].find(')') else {
            result.push_str(&rest[start..]);
            return result;
        };
        result.push_str(&rest[start + 1..start + close_bracket]);
        rest = &rest[after_bracket + close_paren + 1..];
    }

    result.push_str(rest);
    result
}

fn strip_code_fence_markers(text: &str) -> String {
    text.replace("```", " ")
}

fn strip_xml_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }
    result
}

fn is_bad_title(title: &str) -> bool {
    let lower = title.to_lowercase();
    if title.chars().count() <= 2 {
        return true;
    }
    if lower.starts_with("# agents.md instructions")
        || lower.starts_with("agents.md instructions")
        || lower.starts_with("codegraph")
        || lower.starts_with("## codegraph")
        || lower.starts_with("environment_context")
        || lower.starts_with("untrusted evidence")
        || lower.starts_with("command-message")
        || lower.starts_with("system-reminder")
        || lower.starts_with("project-doc")
        || lower.contains("codegraph_start")
        || lower.starts_with('/')
    {
        return true;
    }
    matches!(
        lower.as_str(),
        "<instructions>" | "</instructions>" | "--- project-doc ---"
    )
}

fn remove_prompt_blocks(text: &str) -> String {
    let mut result = text.to_string();
    for tag in ["INSTRUCTIONS", "environment_context"] {
        result = remove_tag_blocks(&result, tag);
    }
    result
}

fn remove_tag_blocks(text: &str, tag: &str) -> String {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut result = String::with_capacity(text.len());
    let mut rest = text;

    loop {
        let Some(start) = rest.find(&open) else {
            result.push_str(rest);
            break;
        };
        result.push_str(&rest[..start]);
        let after_open = start + open.len();
        let Some(end) = rest[after_open..].find(&close) else {
            break;
        };
        rest = &rest[after_open + end + close.len()..];
    }

    result
}

fn extract_between_tags<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = text.find(&open)?;
    let content_start = start + open.len();
    let end = text[content_start..].find(&close)?;
    Some(&text[content_start..content_start + end])
}

fn dedupe_entries(entries: Vec<WorklogEntry>) -> Vec<WorklogEntry> {
    let mut seen_sessions = HashSet::new();
    let mut seen_titles = HashSet::new();
    let mut result = Vec::new();

    for entry in entries {
        if entry.title.trim().is_empty() || is_bad_title(&entry.title) {
            continue;
        }
        let session_key = format!("{}:{}", entry.provider, entry.session_id_full);
        if !seen_sessions.insert(session_key) {
            continue;
        }
        if !seen_titles.insert(entry.title.clone()) {
            continue;
        }
        result.push(entry);
    }

    result
}

fn sort_key(entry: &WorklogEntry) -> String {
    entry
        .first_time
        .as_ref()
        .or(entry.last_time.as_ref())
        .cloned()
        .unwrap_or_else(|| "9999-99-99T99:99:99Z".to_string())
}

fn normalize_time(ts: &str) -> Option<String> {
    if ts.trim().is_empty() {
        None
    } else {
        Some(ts.to_string())
    }
}

fn short_id(id: &str) -> String {
    id.chars().take(8).collect()
}

fn compact_file_name(path: &str) -> String {
    path.trim_start_matches("./").to_string()
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!(
            "{}...",
            s.chars().take(max.saturating_sub(3)).collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SessionMetadata;
    use crate::provider::Provider;

    struct FakeProvider {
        id: &'static str,
        sessions: Vec<Session>,
        messages: Vec<Message>,
    }

    impl Provider for FakeProvider {
        fn id(&self) -> &str {
            self.id
        }

        fn display_name(&self) -> &str {
            self.id
        }

        fn is_available(&self) -> bool {
            true
        }

        fn scan_projects(&self) -> Result<Vec<crate::model::Project>> {
            Ok(Vec::new())
        }

        fn list_sessions(&self, _project: &crate::model::Project) -> Result<Vec<Session>> {
            Ok(self.sessions.clone())
        }

        fn list_all_sessions(&self) -> Result<Vec<Session>> {
            Ok(self.sessions.clone())
        }

        fn load_messages(&self, _session: &Session) -> Result<Vec<Message>> {
            Ok(self.messages.clone())
        }

        fn search(
            &self,
            _opts: &crate::search::SearchOptions,
        ) -> Result<Vec<crate::model::SearchResult>> {
            Ok(Vec::new())
        }
    }

    fn session(provider: &str, id: &str, project: &str, first: &str, last: &str) -> Session {
        session_with_summary(
            provider,
            id,
            project,
            first,
            last,
            "实现 today 工作日志聚合",
        )
    }

    fn session_with_summary(
        provider: &str,
        id: &str,
        project: &str,
        first: &str,
        last: &str,
        summary: &str,
    ) -> Session {
        Session {
            provider: provider.to_string(),
            id: id.to_string(),
            file_path: format!("/tmp/{id}.jsonl"),
            project_name: project.to_string(),
            message_count: 4,
            first_time: first.to_string(),
            last_time: last.to_string(),
            summary: Some(summary.to_string()),
            metadata: None,
            is_subagent: false,
            parent_session_id: None,
            agent_type: None,
            agent_description: None,
        }
    }

    fn user_message(text: &str) -> Message {
        Message {
            role: Role::User,
            timestamp: "2026-06-03T01:00:00Z".to_string(),
            text: text.to_string(),
            tool_name: None,
            tool_input: None,
            tool_output: None,
            model: None,
            thinking: None,
        }
    }

    #[test]
    fn titles_mode_has_title_entries() {
        let report = TodayReport {
            project: "demo".to_string(),
            date: "2026-06-03".to_string(),
            entries: vec![WorklogEntry {
                title: "实现 today 工作日志聚合".to_string(),
                provider: "codex".to_string(),
                session_id: "019e8b96".to_string(),
                session_id_full: "019e8b96-full".to_string(),
                project: "/repo".to_string(),
                first_time: None,
                last_time: None,
                message_count: 2,
                files_touched: Vec::new(),
                summary: Vec::new(),
            }],
        };
        assert_eq!(title_entries(&report)[0].title, "实现 today 工作日志聚合");
    }

    #[test]
    fn default_registry_query_uses_all_providers() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(FakeProvider {
            id: "codex",
            sessions: vec![session_with_summary(
                "codex",
                "codex-session",
                "/repo",
                "2026-06-03T01:00:00Z",
                "2026-06-03T02:00:00Z",
                "Codex work",
            )],
            messages: vec![user_message("Codex work")],
        }));
        registry.register(Box::new(FakeProvider {
            id: "cursor",
            sessions: vec![session_with_summary(
                "cursor",
                "cursor-session",
                "/repo",
                "2026-06-03T03:00:00Z",
                "2026-06-03T04:00:00Z",
                "Cursor work",
            )],
            messages: vec![user_message("Cursor work")],
        }));

        let report = build_today_report(
            &registry,
            &TodayOptions {
                project: Some("/repo"),
                date: Some("2026-06-03"),
            },
            None,
        )
        .unwrap();

        assert_eq!(report.entries.len(), 2);
    }

    #[test]
    fn provider_filter_limits_to_codex() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(FakeProvider {
            id: "codex",
            sessions: vec![session(
                "codex",
                "codex-session",
                "/repo",
                "2026-06-03T01:00:00Z",
                "2026-06-03T02:00:00Z",
            )],
            messages: vec![user_message("Codex work")],
        }));
        registry.register(Box::new(FakeProvider {
            id: "cursor",
            sessions: vec![session(
                "cursor",
                "cursor-session",
                "/repo",
                "2026-06-03T03:00:00Z",
                "2026-06-03T04:00:00Z",
            )],
            messages: vec![user_message("Cursor work")],
        }));

        let filter = vec!["codex".to_string()];
        let report = build_today_report(
            &registry,
            &TodayOptions {
                project: Some("/repo"),
                date: Some("2026-06-03"),
            },
            Some(&filter),
        )
        .unwrap();

        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.entries[0].provider, "codex");
    }

    #[test]
    fn session_crossing_utc_date_is_included_by_local_overlap() {
        let s = session(
            "codex",
            "cross-midnight",
            "/repo",
            "2026-06-02T15:30:00Z",
            "2026-06-02T17:30:00Z",
        );

        assert!(session_overlaps_date(
            &s,
            NaiveDate::from_ymd_opt(2026, 6, 3).unwrap()
        ));
    }

    #[test]
    fn system_prompt_summary_is_not_used_as_title() {
        let mut s = session(
            "codex",
            "system-title",
            "/repo",
            "2026-06-03T01:00:00Z",
            "2026-06-03T02:00:00Z",
        );
        s.summary = Some("# AGENTS.md instructions for /repo".to_string());

        let title = extract_title(&s, &[user_message("修复登录页崩溃")], &[]);
        assert_eq!(title, "修复登录页崩溃");
    }

    #[test]
    fn duplicate_titles_are_deduped() {
        let entries = vec![
            WorklogEntry {
                title: "同一个标题".to_string(),
                provider: "codex".to_string(),
                session_id: "a".to_string(),
                session_id_full: "a-full".to_string(),
                project: "/repo".to_string(),
                first_time: None,
                last_time: None,
                message_count: 2,
                files_touched: Vec::new(),
                summary: Vec::new(),
            },
            WorklogEntry {
                title: "同一个标题".to_string(),
                provider: "cursor".to_string(),
                session_id: "b".to_string(),
                session_id_full: "b-full".to_string(),
                project: "/repo".to_string(),
                first_time: None,
                last_time: None,
                message_count: 2,
                files_touched: Vec::new(),
                summary: Vec::new(),
            },
        ];

        assert_eq!(dedupe_entries(entries).len(), 1);
    }

    #[test]
    fn json_output_shape_is_valid_json() {
        let entries = vec![WorklogEntry {
            title: "实现 today 工作日志聚合".to_string(),
            provider: "codex".to_string(),
            session_id: "019e8b96".to_string(),
            session_id_full: "019e8b96-full".to_string(),
            project: "/repo".to_string(),
            first_time: None,
            last_time: None,
            message_count: 2,
            files_touched: Vec::new(),
            summary: vec!["实现 today 工作日志聚合".to_string()],
        }];

        let json = serde_json::to_string(&entries).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_array());
    }

    #[test]
    fn empty_project_defaults_to_current_directory() {
        let filter = resolve_project_filter(None).unwrap();
        assert_eq!(filter.raw, ".");
        assert!(filter.canonical.is_some());
    }

    #[test]
    fn summary_includes_metadata_files() {
        let mut s = session(
            "codex",
            "metadata",
            "/repo",
            "2026-06-03T01:00:00Z",
            "2026-06-03T02:00:00Z",
        );
        s.metadata = Some(SessionMetadata {
            files_touched: vec!["src/today.rs".to_string()],
            tools_used: vec!["apply_patch".to_string()],
            has_errors: false,
            languages: vec!["rust".to_string()],
        });

        let entry = entry_from_session(&s, &[]);
        assert!(entry
            .summary
            .iter()
            .any(|line| line.contains("src/today.rs")));
    }
}
