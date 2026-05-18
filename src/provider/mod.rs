pub mod claude;
pub mod codex;
pub mod cursor;

use anyhow::Result;
use crate::model::{Message, Project, SearchResult, Session};
use crate::search::SearchOptions;

pub trait Provider: Send + Sync {
    fn id(&self) -> &str;
    fn display_name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn scan_projects(&self) -> Result<Vec<Project>>;
    fn list_sessions(&self, project: &Project) -> Result<Vec<Session>>;
    fn load_messages(&self, session: &Session) -> Result<Vec<Message>>;
    fn search(&self, opts: &SearchOptions) -> Result<Vec<SearchResult>>;
}

pub struct ProviderRegistry {
    providers: Vec<Box<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn Provider>) {
        self.providers.push(provider);
    }

    pub fn available_providers(&self) -> Vec<&dyn Provider> {
        self.providers
            .iter()
            .filter(|p| p.is_available())
            .map(|p| p.as_ref())
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&dyn Provider> {
        self.providers
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref())
    }

    pub fn scan_all_projects(&self, filter: Option<&[String]>) -> Result<Vec<Project>> {
        let mut all = Vec::new();
        for provider in &self.providers {
            if !provider.is_available() {
                continue;
            }
            if let Some(filter) = filter {
                if !filter.iter().any(|f| f == provider.id()) {
                    continue;
                }
            }
            match provider.scan_projects() {
                Ok(projects) => all.extend(projects),
                Err(_) => continue,
            }
        }
        all.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        Ok(all)
    }

    pub fn search_all(&self, opts: &SearchOptions, filter: Option<&[String]>) -> Result<Vec<SearchResult>> {
        let mut all = Vec::new();
        for provider in &self.providers {
            if !provider.is_available() {
                continue;
            }
            if let Some(filter) = filter {
                if !filter.iter().any(|f| f == provider.id()) {
                    continue;
                }
            }
            match provider.search(opts) {
                Ok(results) => all.extend(results),
                Err(_) => continue,
            }
        }
        if opts.sort_by_time {
            all.sort_by(|a, b| b.message.timestamp.cmp(&a.message.timestamp));
        } else {
            all.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        }
        all.truncate(opts.limit);
        Ok(all)
    }

    pub fn find_session(&self, session_id: &str, filter: Option<&[String]>) -> Result<Option<(Session, &dyn Provider)>> {
        let projects = self.scan_all_projects(filter)?;
        for project in &projects {
            let provider = self.get(&project.provider).unwrap();
            if let Ok(sessions) = provider.list_sessions(project) {
                for session in sessions {
                    if session.id == session_id || session.file_path.contains(session_id) {
                        return Ok(Some((session, provider)));
                    }
                }
            }
        }
        Ok(None)
    }
}

pub fn build_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(claude::ClaudeProvider::new()));
    registry.register(Box::new(codex::CodexProvider::new()));
    registry.register(Box::new(cursor::CursorProvider::new()));
    registry
}
