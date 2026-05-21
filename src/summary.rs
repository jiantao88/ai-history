use std::collections::HashMap;

use anyhow::{bail, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::model::{Message, Role, Session};
use crate::provider::ProviderRegistry;

#[derive(Debug, Clone, Serialize)]
pub struct WorkSummary {
    pub date_label: String,
    pub projects: Vec<ProjectSummary>,
    pub total_sessions: usize,
    pub total_messages: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectSummary {
    pub project: String,
    pub sessions: Vec<SessionEntry>,
    pub total_messages: usize,
    pub active_time_minutes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionEntry {
    pub id: String,
    pub project: String,
    pub time_start: String,
    pub time_end: String,
    pub message_count: usize,
    pub model: Option<String>,
    pub work_type: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

pub fn parse_date_range(
    date: Option<&str>,
    range: Option<&str>,
    today: bool,
) -> Result<DateRange> {
    if let Some(range_str) = range {
        let parts: Vec<&str> = range_str.split("..").collect();
        if parts.len() != 2 {
            bail!("Invalid range format. Use YYYY-MM-DD..YYYY-MM-DD");
        }
        let start = NaiveDate::parse_from_str(parts[0], "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid start date: {}", parts[0]))?;
        let end = NaiveDate::parse_from_str(parts[1], "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid end date: {}", parts[1]))?;
        return Ok(DateRange { start, end });
    }

    if let Some(date_str) = date {
        let d = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid date: {}", date_str))?;
        return Ok(DateRange { start: d, end: d });
    }

    // Default: today
    let _ = today;
    let now = chrono::Local::now().date_naive();
    Ok(DateRange {
        start: now,
        end: now,
    })
}

fn session_date(session: &Session) -> Option<NaiveDate> {
    chrono::DateTime::parse_from_rfc3339(&session.first_time)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Local).date_naive())
}

fn in_date_range(session: &Session, range: &DateRange) -> bool {
    session_date(session)
        .map(|d| d >= range.start && d <= range.end)
        .unwrap_or(false)
}

fn extract_model(messages: &[Message]) -> Option<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for msg in messages {
        if let Some(ref model) = msg.model {
            *counts.entry(model.clone()).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(model, _)| model)
}

fn classify_work_type(first_user_msg: &str, messages: &[Message]) -> String {
    if let Some(cmd) = extract_between_tags(first_user_msg, "command-message") {
        if cmd.contains("review") {
            return "代码审查".to_string();
        }
    }

    let early_user_text: String = messages
        .iter()
        .filter(|m| matches!(m.role, Role::User))
        .take(3)
        .map(|m| strip_xml_tags(&m.text).to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");

    let stripped = early_user_text;

    if stripped.contains("review") || stripped.contains("pr ") || stripped.contains("审查") {
        return "代码审查".to_string();
    }

    let has_tool_edits = messages.iter().any(|m| {
        m.tool_name
            .as_deref()
            .map(|t| matches!(t, "Write" | "Edit" | "MultiEdit"))
            .unwrap_or(false)
    });

    if stripped.contains("fix ") || stripped.contains("bug") || stripped.contains("error")
        || stripped.contains("修复") || stripped.contains("报错")
    {
        return "bug修复".to_string();
    }

    if stripped.contains("refactor") || stripped.contains("重构") || stripped.contains("rename") {
        return "重构".to_string();
    }

    if stripped.contains("optimi") || stripped.contains("perf") || stripped.contains("优化")
        || stripped.contains("improve") || stripped.contains("speed") || stripped.contains("faster")
        || stripped.contains("替换") || stripped.contains("替代") || stripped.contains("改进")
    {
        return "优化".to_string();
    }

    if stripped.contains("add ") || stripped.contains("implement") || stripped.contains("create")
        || stripped.contains("new ") || stripped.contains("feature")
        || stripped.contains("新增") || stripped.contains("添加") || stripped.contains("实现")
        || stripped.contains("需求") || stripped.contains("功能")
    {
        return "新功能".to_string();
    }

    if has_tool_edits {
        return "开发".to_string();
    }

    "其他".to_string()
}

fn extract_summary_text(first_user_msg: &str) -> String {
    if let Some(args) = extract_between_tags(first_user_msg, "command-args") {
        let cleaned = args.trim();
        if !cleaned.is_empty() && cleaned.len() > 3 {
            return truncate_summary(cleaned);
        }
    }

    if let Some(cmd) = extract_between_tags(first_user_msg, "command-message") {
        let cmd = cmd.trim();
        if cmd.contains("review") {
            return "Code Review".to_string();
        }
    }

    let text = first_user_msg
        .lines()
        .map(|l| l.trim())
        .find(|l| {
            !l.is_empty()
                && !l.starts_with('<')
                && !l.starts_with('/')
                && !l.starts_with("command")
                && l.len() > 3
        })
        .unwrap_or("");

    let cleaned = strip_xml_tags(text);
    let cleaned = cleaned
        .replace("[Image #1]", "")
        .replace("[Image #2]", "")
        .replace("[Image #3]", "");
    let cleaned = cleaned.trim();

    if cleaned.is_empty() || cleaned.len() <= 3 {
        return "-".to_string();
    }

    truncate_summary(cleaned)
}

fn truncate_summary(s: &str) -> String {
    if s.chars().count() <= 60 {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(57).collect();
        format!("{}...", truncated)
    }
}

fn extract_between_tags<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = text.find(&open)?;
    let content_start = start + open.len();
    let end = text[content_start..].find(&close)?;
    Some(&text[content_start..content_start + end])
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

fn format_session_time(ts: &str, range: &DateRange) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
        let local = dt.with_timezone(&chrono::Local);
        let date = local.date_naive();
        let single_day = range.start == range.end;
        if single_day && date == range.start {
            local.format("%H:%M").to_string()
        } else {
            local.format("%m-%d %H:%M").to_string()
        }
    } else if ts.len() >= 16 {
        ts[11..16].to_string()
    } else {
        ts.to_string()
    }
}

pub fn build_summary(
    registry: &ProviderRegistry,
    project_filter: Option<&str>,
    date_range: &DateRange,
    provider_filter: Option<&[String]>,
    use_ai: bool,
) -> Result<WorkSummary> {
    let projects = registry.scan_all_projects(provider_filter)?;

    let filtered_projects: Vec<_> = if let Some(pf) = project_filter {
        projects
            .into_iter()
            .filter(|p| p.name.contains(pf) || p.path.contains(pf))
            .collect()
    } else {
        projects
    };

    let mut all_entries: Vec<SessionEntry> = Vec::new();
    let mut all_contexts: Vec<String> = Vec::new();

    for project in &filtered_projects {
        let provider = registry.get(&project.provider).unwrap();
        let sessions = match provider.list_sessions(project) {
            Ok(s) => s,
            Err(_) => continue,
        };

        for session in sessions {
            if session.is_subagent {
                continue;
            }
            if !in_date_range(&session, date_range) {
                continue;
            }
            if session.message_count <= 1 {
                continue;
            }

            let messages = match provider.load_messages(&session) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let first_user_msg = messages
                .iter()
                .find(|m| matches!(m.role, Role::User) && !m.text.trim().is_empty())
                .map(|m| m.text.as_str())
                .unwrap_or("");

            let model = extract_model(&messages);
            let work_type = classify_work_type(first_user_msg, &messages);
            let summary = extract_summary_text(first_user_msg);

            let time_start = format_session_time(&session.first_time, date_range);
            let time_end = format_session_time(&session.last_time, date_range);

            all_entries.push(SessionEntry {
                id: if session.id.len() > 8 {
                    session.id[..8].to_string()
                } else {
                    session.id.clone()
                },
                project: session.project_name.clone(),
                time_start,
                time_end,
                message_count: session.message_count,
                model,
                work_type,
                summary,
            });

            if use_ai {
                all_contexts.push(build_session_context(&messages));
            }
        }
    }

    if use_ai && !all_entries.is_empty() {
        match enhance_with_llm(&mut all_entries, &all_contexts) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Warning: AI summary failed ({e}), using rule-based results");
            }
        }
    }

    all_entries.sort_by(|a, b| a.time_start.cmp(&b.time_start));

    let mut project_map: HashMap<String, Vec<SessionEntry>> = HashMap::new();
    for entry in &all_entries {
        project_map
            .entry(entry.project.clone())
            .or_default()
            .push(entry.clone());
    }

    let mut project_summaries: Vec<ProjectSummary> = project_map
        .into_iter()
        .map(|(project, sessions)| {
            let total_messages: usize = sessions.iter().map(|s| s.message_count).sum();
            let active_time = estimate_active_minutes(&sessions);
            ProjectSummary {
                project,
                sessions,
                total_messages,
                active_time_minutes: active_time,
            }
        })
        .collect();
    project_summaries.sort_by(|a, b| b.total_messages.cmp(&a.total_messages));

    let total_sessions = all_entries.len();
    let total_messages: usize = all_entries.iter().map(|e| e.message_count).sum();

    let date_label = if date_range.start == date_range.end {
        date_range.start.format("%Y-%m-%d").to_string()
    } else {
        format!(
            "{} ~ {}",
            date_range.start.format("%Y-%m-%d"),
            date_range.end.format("%Y-%m-%d")
        )
    };

    Ok(WorkSummary {
        date_label,
        projects: project_summaries,
        total_sessions,
        total_messages,
    })
}

fn estimate_active_minutes(sessions: &[SessionEntry]) -> u64 {
    let mut total: u64 = 0;
    for s in sessions {
        let start_mins = parse_time_to_mins(&s.time_start);
        let end_mins = parse_time_to_mins(&s.time_end);
        if let (Some(start), Some(end)) = (start_mins, end_mins) {
            if end >= start {
                total += end - start;
            } else {
                total += (24 * 60 - start) + end;
            }
        }
    }
    total
}

fn parse_time_to_mins(t: &str) -> Option<u64> {
    let hm = if t.contains(' ') {
        t.rsplit(' ').next()?
    } else {
        t
    };
    let parts: Vec<&str> = hm.split(':').collect();
    if parts.len() == 2 {
        let h: u64 = parts[0].parse().ok()?;
        let m: u64 = parts[1].parse().ok()?;
        Some(h * 60 + m)
    } else {
        None
    }
}

#[derive(Debug, Deserialize)]
struct LlmSessionResult {
    summary: String,
    work_type: String,
}

#[derive(Debug, Deserialize)]
struct LlmBatchResult {
    sessions: Vec<LlmSessionResult>,
}

fn build_session_context(messages: &[Message]) -> String {
    let mut out = String::new();
    let mut char_count = 0;
    let max_chars = 2000;

    let head: Vec<_> = messages
        .iter()
        .filter(|m| matches!(m.role, Role::User | Role::Assistant) && !m.text.trim().is_empty())
        .take(4)
        .collect();

    let tail: Vec<_> = messages
        .iter()
        .rev()
        .filter(|m| matches!(m.role, Role::User | Role::Assistant) && !m.text.trim().is_empty())
        .take(2)
        .collect();

    for msg in &head {
        let role = if matches!(msg.role, Role::User) { "U" } else { "A" };
        let text = strip_xml_tags(&msg.text);
        let text: String = text.chars().take(400).collect();
        let line = format!("{}: {}\n", role, text.trim());
        char_count += line.len();
        if char_count > max_chars {
            break;
        }
        out.push_str(&line);
    }

    for msg in tail.iter().rev() {
        let role = if matches!(msg.role, Role::User) { "U" } else { "A" };
        let text = strip_xml_tags(&msg.text);
        let text: String = text.chars().take(300).collect();
        let line = format!("{}: {}\n", role, text.trim());
        char_count += line.len();
        if char_count > max_chars {
            break;
        }
        out.push_str(&line);
    }

    out
}

pub fn enhance_with_llm(entries: &mut [SessionEntry], session_contexts: &[String]) -> Result<()> {
    let api_key = std::env::var("ANTHROPIC_AUTH_TOKEN")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN not set."))?;

    let use_bearer = std::env::var("ANTHROPIC_AUTH_TOKEN").is_ok();

    let base_url = std::env::var("ANTHROPIC_BASE_URL")
        .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
    let api_url = format!("{}/v1/messages", base_url.trim_end_matches('/'));

    let mut batch_input = String::new();
    for (i, ctx) in session_contexts.iter().enumerate() {
        batch_input.push_str(&format!(
            "--- Session {} (rule: type={}, summary={}) ---\n{}\n",
            i + 1,
            entries[i].work_type,
            entries[i].summary,
            ctx,
        ));
    }

    let work_types = "bug修复, 优化, 新功能, 重构, 代码审查, 开发, 其他";

    let prompt = format!(
        "Analyze these {count} AI coding sessions and generate a one-line Chinese summary and work type for each.\n\n\
         Work types (pick exactly one): {work_types}\n\n\
         {batch_input}\n\n\
         Output ONLY valid JSON, no other text:\n\
         {{\"sessions\": [{{\"summary\": \"一行中文摘要\", \"work_type\": \"类型\"}}]}}\n\
         Rules:\n\
         - summary should be concise (under 40 chars), describing what was done, in Chinese\n\
         - work_type must be one of the listed types\n\
         - Return exactly {count} items in the same order",
        count = entries.len(),
        work_types = work_types,
        batch_input = batch_input,
    );

    let model = std::env::var("ANTHROPIC_MODEL")
        .unwrap_or_else(|_| "claude-haiku-4-5-20251001".to_string());

    let client = reqwest::blocking::Client::new();
    let mut req = client
        .post(&api_url)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json");

    if use_bearer {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    } else {
        req = req.header("x-api-key", &api_key);
    }

    let response = req
        .json(&serde_json::json!({
            "model": model,
            "max_tokens": 1024,
            "messages": [{"role": "user", "content": prompt}]
        }))
        .send()?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("API error {status}: {body}");
    }

    let resp_json: serde_json::Value = response.json()?;
    let content_text = resp_json["content"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Unexpected API response format"))?;

    let json_text = strip_code_fence(content_text);

    let batch_result: LlmBatchResult = serde_json::from_str(&json_text)
        .map_err(|e| anyhow::anyhow!("Failed to parse LLM JSON: {e}\nRaw: {content_text}"))?;

    for (i, llm_entry) in batch_result.sessions.iter().enumerate() {
        if i >= entries.len() {
            break;
        }
        if !llm_entry.summary.is_empty() {
            entries[i].summary = truncate_summary(&llm_entry.summary);
        }
        if !llm_entry.work_type.is_empty() {
            entries[i].work_type = llm_entry.work_type.clone();
        }
    }

    Ok(())
}

fn strip_code_fence(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.starts_with("```") {
        let without_open = if let Some(pos) = trimmed.find('\n') {
            &trimmed[pos + 1..]
        } else {
            trimmed.trim_start_matches('`')
        };
        let without_close = without_open.trim_end().trim_end_matches("```").trim();
        without_close.to_string()
    } else {
        trimmed.to_string()
    }
}
