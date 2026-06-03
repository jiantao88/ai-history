use crate::model::{Message, Project, SearchResult, Session};
use crate::summary::WorkSummary;
use crate::today::{WorklogEntry, WorklogTitle};
use crate::workflows::WorkflowReport;

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

pub fn print_today_entries(entries: &[WorklogEntry]) {
    println!("{}", serde_json::to_string_pretty(entries).unwrap());
}

pub fn print_today_titles(titles: &[WorklogTitle]) {
    println!("{}", serde_json::to_string_pretty(titles).unwrap());
}

pub fn print_workflow_report(report: &WorkflowReport) {
    println!("{}", serde_json::to_string_pretty(report).unwrap());
}
