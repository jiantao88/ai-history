#![allow(dead_code)]

mod cli;
mod digest;
mod model;
mod output;
mod parse;
mod provider;
mod scoring;
mod search;
mod summary;

use anyhow::{bail, Result};
use clap::Parser;

use cli::{Cli, Command, ExportFormat};
use output::is_tty;
use provider::build_registry;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let registry = build_registry();
    let filter = cli.provider.as_deref();
    let use_json = cli.json || !is_tty();

    match cli.command {
        Command::List => {
            let projects = registry.scan_all_projects(filter)?;
            if use_json {
                output::json::print_projects(&projects);
            } else {
                output::human::print_projects(&projects);
            }
        }

        Command::Sessions { project, no_subagents } => {
            let projects = registry.scan_all_projects(filter)?;
            let matched = projects
                .iter()
                .find(|p| p.name.contains(&project) || p.path.contains(&project));

            let Some(matched) = matched else {
                bail!("Project not found: {project}");
            };

            let provider = registry.get(&matched.provider).unwrap();
            let mut sessions = provider.list_sessions(matched)?;

            if no_subagents {
                sessions.retain(|s| !s.is_subagent);
            }

            if use_json {
                output::json::print_sessions(&sessions);
            } else {
                output::human::print_sessions(&sessions);
            }
        }

        Command::Show { session, compact } => {
            let (found, provider) = registry
                .find_session(&session, filter)?
                .ok_or_else(|| anyhow::anyhow!("Session not found: {session}"))?;

            let messages = provider.load_messages(&found)?;
            let messages = if compact {
                messages
                    .into_iter()
                    .filter(|m| matches!(m.role, model::Role::User | model::Role::Assistant))
                    .collect()
            } else {
                messages
            };

            if use_json {
                output::json::print_messages(&messages);
            } else if is_tty() {
                output::human::print_messages(&messages, false);
            } else {
                output::human::print_messages_plain(&messages, false);
            }
        }

        Command::Search { query, limit, context_window, require_all, sort_by_time } => {
            let opts = search::SearchOptions {
                query,
                limit,
                context_window,
                require_all_terms: require_all,
                sort_by_time,
            };
            let results = search::search_all(&registry, &opts, filter)?;

            if use_json {
                output::json::print_search_results(&results);
            } else {
                output::human::print_search_results(&results);
            }
        }

        Command::Export { session, format } => {
            let (found, provider) = registry
                .find_session(&session, filter)?
                .ok_or_else(|| anyhow::anyhow!("Session not found: {session}"))?;

            let messages = provider.load_messages(&found)?;

            match format {
                ExportFormat::Md => {
                    print!("{}", output::markdown::export_session(&found, &messages));
                }
                ExportFormat::Json => {
                    output::json::print_messages(&messages);
                }
                ExportFormat::Prompt => {
                    print!("{}", output::prompt::export_session(&found, &messages));
                }
            }
        }

        Command::Context { session, full, llm } => {
            let (found, provider) = registry
                .find_session(&session, filter)?
                .ok_or_else(|| anyhow::anyhow!("Session not found: {session}"))?;

            let messages = provider.load_messages(&found)?;

            if full {
                print!("{}", output::prompt::export_session(&found, &messages));
            } else {
                let d = digest::get_or_create_digest(&found, &messages, llm, false)?;
                print!("{}", digest::format_digest(&d));
            }
        }

        Command::Digest { session, llm, no_cache } => {
            let (found, provider) = registry
                .find_session(&session, filter)?
                .ok_or_else(|| anyhow::anyhow!("Session not found: {session}"))?;

            let messages = provider.load_messages(&found)?;
            let d = digest::get_or_create_digest(&found, &messages, llm, no_cache)?;

            if use_json {
                println!("{}", serde_json::to_string_pretty(&d).unwrap());
            } else {
                print!("{}", digest::format_digest(&d));
            }
        }

        Command::Summary { project, date, range, today, ai_summary } => {
            let date_range = summary::parse_date_range(
                date.as_deref(),
                range.as_deref(),
                today,
            )?;

            let result = summary::build_summary(
                &registry,
                project.as_deref(),
                &date_range,
                filter,
                ai_summary,
            )?;

            if use_json {
                output::json::print_summary(&result);
            } else if is_tty() {
                output::human::print_summary(&result);
            } else {
                output::human::print_summary_plain(&result);
            }
        }
    }

    Ok(())
}
