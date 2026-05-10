#![allow(dead_code)]

mod cli;
mod model;
mod output;
mod parse;
mod provider;
mod scoring;
mod search;

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

        Command::Sessions { project } => {
            let projects = registry.scan_all_projects(filter)?;
            let matched = projects
                .iter()
                .find(|p| p.name.contains(&project) || p.path.contains(&project));

            let Some(matched) = matched else {
                bail!("Project not found: {project}");
            };

            let provider = registry.get(&matched.provider).unwrap();
            let sessions = provider.list_sessions(matched)?;

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
    }

    Ok(())
}
