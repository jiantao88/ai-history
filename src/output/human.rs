use crate::model::{Message, Project, Role, SearchResult, Session, SessionMetadata};
use crate::summary::WorkSummary;
use crate::today::TodayReport;
use crate::workflows::WorkflowReport;
use owo_colors::OwoColorize;

pub fn print_projects(projects: &[Project]) {
    if projects.is_empty() {
        println!("{}", "No projects found.".dimmed());
        return;
    }

    println!(
        "{:<10} {:<40} {:>8} {}",
        "Provider".bold(),
        "Project".bold(),
        "Sessions".bold(),
        "Last Active".bold(),
    );
    println!("{}", "─".repeat(80));

    for p in projects {
        println!(
            "{:<10} {:<40} {:>8} {}",
            p.provider.cyan(),
            truncate(&p.name, 40),
            p.session_count,
            format_time(&p.last_modified).dimmed(),
        );
    }
}

pub fn print_sessions(sessions: &[Session]) {
    if sessions.is_empty() {
        println!("{}", "No sessions found.".dimmed());
        return;
    }

    println!(
        "{:<36} {:>5} {:<20} {:<40} {}",
        "Session ID".bold(),
        "Msgs".bold(),
        "Date".bold(),
        "Summary".bold(),
        "Tags".bold(),
    );
    println!("{}", "─".repeat(120));

    for s in sessions {
        let summary = s.summary.as_deref().unwrap_or("-");
        let mut tags = format_metadata_tags(&s.metadata);
        if s.is_subagent {
            let agent_tag = s.agent_type.as_deref().unwrap_or("subagent");
            if tags.is_empty() {
                tags = format!("[{}]", agent_tag);
            } else {
                tags = format!("[{}] {}", agent_tag, tags);
            }
        }
        println!(
            "{:<36} {:>5} {:<20} {:<40} {}",
            truncate(&s.id, 36).dimmed(),
            s.message_count,
            format_time(&s.first_time),
            truncate(summary, 40),
            tags.dimmed(),
        );
    }
}

pub fn print_messages(messages: &[Message], compact: bool) {
    for msg in messages {
        if compact && !matches!(msg.role, Role::User | Role::Assistant) {
            continue;
        }

        let role_label = match msg.role {
            Role::User => "User".green().bold().to_string(),
            Role::Assistant => "Assistant".blue().bold().to_string(),
            Role::System => "System".yellow().to_string(),
            Role::Tool => "Tool".dimmed().to_string(),
        };

        if let Some(ref thinking) = msg.thinking {
            println!("\n{}", "─ thinking ─".dimmed().italic());
            println!("{}", thinking.dimmed().italic());
        }

        println!("\n{} {}", role_label, format_time(&msg.timestamp).dimmed());

        if let Some(ref tool) = msg.tool_name {
            print!("  {} {}", "tool:".dimmed(), tool.yellow());
            if let Some(ref input) = msg.tool_input {
                println!("\n{}", indent(input, "    ").dimmed());
            } else {
                println!();
            }
            if let Some(ref output) = msg.tool_output {
                println!("{}", indent(&truncate(output, 500), "    ").dimmed());
            }
        }

        if !msg.text.is_empty() {
            println!("{}", msg.text);
        }
    }
}

pub fn print_search_results(results: &[SearchResult]) {
    if results.is_empty() {
        println!("{}", "No results found.".dimmed());
        return;
    }

    for (i, r) in results.iter().enumerate() {
        let score_str = if r.score > 0.0 {
            format!("  score: {:.2}", r.score)
        } else {
            String::new()
        };
        println!(
            "\n{} {} {} {}{}",
            format!("[{}]", i + 1).bold(),
            r.provider.cyan(),
            r.project_name,
            format_time(&r.message.timestamp).dimmed(),
            score_str.yellow(),
        );

        // Context before
        for ctx in &r.context_before {
            let role_tag = format_role_tag(&ctx.role);
            let text = truncate(&ctx.text, 120);
            println!("  {} {} {}", "┊".dimmed(), role_tag.dimmed(), text.dimmed());
        }

        // Matched message (highlighted)
        let preview = truncate(&r.message.text, 200);
        if !r.context_before.is_empty() || !r.context_after.is_empty() {
            let role_tag = format_role_tag(&r.message.role);
            println!("  {} {} {}", "▶".bold().green(), role_tag, preview);
        } else {
            println!("  {}", preview);
        }

        // Context after
        for ctx in &r.context_after {
            let role_tag = format_role_tag(&ctx.role);
            let text = truncate(&ctx.text, 120);
            println!("  {} {} {}", "┊".dimmed(), role_tag.dimmed(), text.dimmed());
        }
    }
}

pub fn print_projects_plain(projects: &[Project]) {
    for p in projects {
        println!(
            "{}\t{}\t{}\t{}",
            p.provider, p.name, p.session_count, p.last_modified,
        );
    }
}

pub fn print_sessions_plain(sessions: &[Session]) {
    for s in sessions {
        println!(
            "{}\t{}\t{}\t{}",
            s.id,
            s.message_count,
            s.first_time,
            s.summary.as_deref().unwrap_or("-"),
        );
    }
}

pub fn print_messages_plain(messages: &[Message], compact: bool) {
    for msg in messages {
        if compact && !matches!(msg.role, Role::User | Role::Assistant) {
            continue;
        }
        println!("[{}] {}", msg.role, msg.text);
    }
}

pub fn print_search_results_plain(results: &[SearchResult]) {
    for r in results {
        println!(
            "{}\t{}\t{}\t{}",
            r.provider,
            r.project_name,
            r.message.timestamp,
            truncate(&r.message.text, 200),
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}...")
    }
}

fn format_time(ts: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else if ts.len() >= 16 {
        ts[..16].to_string()
    } else {
        ts.to_string()
    }
}

fn indent(text: &str, prefix: &str) -> String {
    text.lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_role_tag(role: &Role) -> String {
    match role {
        Role::User => "[User]".to_string(),
        Role::Assistant => "[Assistant]".to_string(),
        Role::System => "[System]".to_string(),
        Role::Tool => "[Tool]".to_string(),
    }
}

pub fn print_summary(summary: &WorkSummary) {
    println!(
        "\n{}",
        format!("AI WORK SUMMARY — {}", summary.date_label)
            .bold()
            .cyan()
    );
    println!("{}", "═".repeat(70));
    println!(
        "Sessions: {}    Messages: {}",
        summary.total_sessions.to_string().bold(),
        summary.total_messages.to_string().bold(),
    );

    let total_active: u64 = summary.projects.iter().map(|p| p.active_time_minutes).sum();
    if total_active > 0 {
        let hours = total_active / 60;
        let mins = total_active % 60;
        if hours > 0 {
            print!("    Active time: ~{}h {}m", hours, mins);
        } else {
            print!("    Active time: ~{}m", mins);
        }
    }
    println!();

    let multi_project = summary.projects.len() > 1;

    println!(
        "{}",
        "───────────────────────────────────────────────────────────────────────"
    );
    if multi_project {
        println!(
            "{:<4} {:<16} {:>5}  {:<8} {:<30} {}",
            "#".bold(),
            "Time".bold(),
            "Msgs".bold(),
            "Type".bold(),
            "Summary".bold(),
            "Project".bold(),
        );
    } else {
        if let Some(first) = summary.projects.first() {
            println!("{}", format!("Project: {}", first.project).dimmed());
        }
        println!(
            "{:<4} {:<16} {:>5}  {:<8} {}",
            "#".bold(),
            "Time".bold(),
            "Msgs".bold(),
            "Type".bold(),
            "Summary".bold(),
        );
    }
    println!(
        "{}",
        "───────────────────────────────────────────────────────────────────────"
    );

    let mut idx = 1;
    for proj in &summary.projects {
        for entry in &proj.sessions {
            let time_range = format!("{}-{}", entry.time_start, entry.time_end);
            let type_colored = match entry.work_type.as_str() {
                "bug修复" => entry.work_type.red().to_string(),
                "新功能" => entry.work_type.green().to_string(),
                "优化" => entry.work_type.yellow().to_string(),
                "重构" => entry.work_type.blue().to_string(),
                "代码审查" => entry.work_type.magenta().to_string(),
                _ => entry.work_type.dimmed().to_string(),
            };
            if multi_project {
                let proj_short = truncate(&entry.project, 20);
                println!(
                    "{:<4} {:<16} {:>5}  {:<8} {:<30} {}",
                    idx,
                    time_range,
                    entry.message_count,
                    type_colored,
                    truncate(&entry.summary, 30),
                    proj_short.dimmed(),
                );
            } else {
                println!(
                    "{:<4} {:<16} {:>5}  {:<8} {}",
                    idx,
                    time_range,
                    entry.message_count,
                    type_colored,
                    truncate(&entry.summary, 45),
                );
            }
            idx += 1;
        }
    }
    println!("{}", "═".repeat(70));
}

pub fn print_summary_plain(summary: &WorkSummary) {
    for proj in &summary.projects {
        for entry in &proj.sessions {
            println!(
                "{}\t{}-{}\t{}\t{}\t{}\t{}",
                entry.id,
                entry.time_start,
                entry.time_end,
                entry.message_count,
                entry.work_type,
                entry.summary,
                entry.project,
            );
        }
    }
}

pub fn print_today_titles(report: &TodayReport) {
    println!(
        "\n{}",
        format!("TODAY WORK TITLES — {}", report.project)
            .bold()
            .cyan()
    );
    println!("{}", "═".repeat(70));

    if report.entries.is_empty() {
        println!("{}", "No work sessions found.".dimmed());
    } else {
        for entry in &report.entries {
            println!("- {}", entry.title);
        }
    }

    println!("{}", "═".repeat(70));
}

pub fn print_today_summary(report: &TodayReport) {
    println!(
        "\n{}",
        format!("TODAY WORK SUMMARY — {}", report.project)
            .bold()
            .cyan()
    );
    println!("{}", "═".repeat(70));

    if report.entries.is_empty() {
        println!("{}", "No work sessions found.".dimmed());
        println!("{}", "═".repeat(70));
        return;
    }

    for (idx, entry) in report.entries.iter().enumerate() {
        println!("\n{}. {}", idx + 1, entry.title.bold());
        println!("   Provider: {}", entry.provider.cyan());
        println!("   Session: {}", entry.session_id.dimmed());
        let time = format_today_time(entry.first_time.as_deref(), entry.last_time.as_deref());
        if !time.is_empty() {
            println!("   Time: {time}");
        }
        if !entry.files_touched.is_empty() {
            println!("   Files:");
            for file in entry.files_touched.iter().take(8) {
                println!("   - {file}");
            }
        }
        if !entry.summary.is_empty() {
            println!("   Summary:");
            for line in &entry.summary {
                println!("   - {line}");
            }
        }
    }

    println!("{}", "═".repeat(70));
}

pub fn print_today_summary_plain(report: &TodayReport) {
    for entry in &report.entries {
        let time = format_today_time(entry.first_time.as_deref(), entry.last_time.as_deref());
        println!(
            "{}\t{}\t{}\t{}\t{}",
            entry.session_id, entry.provider, time, entry.title, entry.project,
        );
    }
}

pub fn print_workflow_report(report: &WorkflowReport) {
    println!(
        "\n{}",
        format!("AI WORKFLOW CANDIDATES — {}", report.date_label)
            .bold()
            .cyan()
    );
    println!("{}", "═".repeat(90));
    println!(
        "Reviewed sessions: {}    Candidates: {}",
        report.total_sessions_reviewed.to_string().bold(),
        report.candidates.len().to_string().bold(),
    );

    if report.candidates.is_empty() {
        println!("{}", "No repeated workflow candidates found.".dimmed());
        return;
    }

    println!(
        "{}",
        "──────────────────────────────────────────────────────────────────────────────"
    );
    println!(
        "{:<3} {:<38} {:>4} {:<10} {:<16} {}",
        "#".bold(),
        "Candidate".bold(),
        "Freq".bold(),
        "Confidence".bold(),
        "Form".bold(),
        "Coverage".bold(),
    );

    for (idx, candidate) in report.candidates.iter().enumerate() {
        let form = if candidate.worth_creating {
            candidate.recommended_form.green().bold().to_string()
        } else {
            candidate.recommended_form.dimmed().to_string()
        };
        let coverage = if candidate.coverage == "missing" {
            "missing".yellow().to_string()
        } else {
            candidate.coverage.dimmed().to_string()
        };
        println!(
            "{:<3} {:<38} {:>4} {:<10} {:<16} {}",
            idx + 1,
            truncate(&candidate.id, 38),
            candidate.frequency,
            candidate.confidence,
            form,
            truncate(&coverage, 32),
        );
        println!("    {}", truncate(&candidate.workflow, 110));
        println!("    {}", candidate.rationale.dimmed());
        let evidence = candidate
            .evidence
            .iter()
            .take(3)
            .map(|e| {
                format!(
                    "{} {} {} {}",
                    e.date,
                    e.provider,
                    e.session_id,
                    truncate(&e.summary, 42)
                )
            })
            .collect::<Vec<_>>()
            .join(" | ");
        if !evidence.is_empty() {
            println!("    {} {}", "Evidence:".bold(), evidence);
        }
        if candidate.worth_creating {
            println!(
                "    {} ai-history workflows --write-skills --skill {}",
                "To write draft:".bold(),
                candidate.id.cyan()
            );
        }
    }

    if !report.written_skills.is_empty() {
        println!(
            "{}",
            "──────────────────────────────────────────────────────────────────────────────"
        );
        println!("{}", "Written skill drafts".bold());
        for skill in &report.written_skills {
            println!("- {} -> {}", skill.skill_name.green(), skill.path.dimmed());
        }
    }

    println!("{}", "═".repeat(90));
}

pub fn print_workflow_report_plain(report: &WorkflowReport) {
    for candidate in &report.candidates {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            candidate.id,
            candidate.frequency,
            candidate.confidence,
            candidate.recommended_form,
            candidate.coverage,
            candidate.worth_creating,
            candidate.workflow,
        );
    }
    for skill in &report.written_skills {
        println!(
            "written\t{}\t{}\t{}",
            skill.candidate_id, skill.skill_name, skill.path,
        );
    }
}

fn format_metadata_tags(metadata: &Option<SessionMetadata>) -> String {
    let Some(meta) = metadata else {
        return String::new();
    };

    let mut parts = Vec::new();

    for lang in &meta.languages {
        parts.push(lang.clone());
    }

    if !meta.files_touched.is_empty() {
        parts.push(format!("{}files", meta.files_touched.len()));
    }

    if meta.has_errors {
        parts.push("⚠err".to_string());
    }

    parts.join(" ")
}

fn format_today_time(first: Option<&str>, last: Option<&str>) -> String {
    let start = first.map(format_today_timestamp).unwrap_or_default();
    let end = last.map(format_today_timestamp).unwrap_or_default();
    if start.is_empty() {
        end
    } else if end.is_empty() || end == start {
        start
    } else {
        format!("{start} - {end}")
    }
}

fn format_today_timestamp(ts: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
        dt.with_timezone(&chrono::Local).format("%H:%M").to_string()
    } else if ts.len() >= 16 {
        ts[11..16].to_string()
    } else {
        ts.to_string()
    }
}
