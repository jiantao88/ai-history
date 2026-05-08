use crate::model::{Message, Project, Role, SearchResult, Session};
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
        "{:<36} {:>5} {:<20} {}",
        "Session ID".bold(),
        "Msgs".bold(),
        "Date".bold(),
        "Summary".bold(),
    );
    println!("{}", "─".repeat(90));

    for s in sessions {
        let summary = s.summary.as_deref().unwrap_or("-");
        println!(
            "{:<36} {:>5} {:<20} {}",
            truncate(&s.id, 36).dimmed(),
            s.message_count,
            format_time(&s.first_time),
            truncate(summary, 40),
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
        println!(
            "\n{} {} {} {}",
            format!("[{}]", i + 1).bold(),
            r.provider.cyan(),
            r.project_name,
            format_time(&r.message.timestamp).dimmed(),
        );
        let preview = truncate(&r.message.text, 200);
        println!("  {}", preview);
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
