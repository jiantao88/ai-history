use crate::model::{Message, Project, SearchResult, Session};
use crate::summary::WorkSummary;

pub fn print_projects(projects: &[Project]) {
    println!("{}", serde_json::to_string_pretty(projects).unwrap());
}

pub fn print_sessions(sessions: &[Session]) {
    println!("{}", serde_json::to_string_pretty(sessions).unwrap());
}

pub fn print_messages(messages: &[Message]) {
    println!("{}", serde_json::to_string_pretty(messages).unwrap());
}

pub fn print_search_results(results: &[SearchResult]) {
    println!("{}", serde_json::to_string_pretty(results).unwrap());
}

pub fn print_summary(summary: &WorkSummary) {
    println!("{}", serde_json::to_string_pretty(summary).unwrap());
}
