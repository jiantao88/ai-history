pub mod cache;
pub mod extractor;
pub mod llm;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::model::{Message, Session};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDigest {
    pub session_id: String,
    pub provider: String,
    pub project_name: String,
    pub time_range: String,
    pub intent: String,
    pub key_decisions: Vec<Decision>,
    pub code_changes: Vec<FileChange>,
    pub remaining_issues: Vec<String>,
    pub conclusion: String,
    pub tools_used: Vec<String>,
    pub had_errors: bool,
    pub llm_enhanced: bool,
    pub source_mtime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub file_path: String,
    pub action: FileAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileAction {
    Created,
    Modified,
    Deleted,
}

impl std::fmt::Display for FileAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileAction::Created => write!(f, "Created"),
            FileAction::Modified => write!(f, "Modified"),
            FileAction::Deleted => write!(f, "Deleted"),
        }
    }
}

pub fn get_or_create_digest(
    session: &Session,
    messages: &[Message],
    use_llm: bool,
    no_cache: bool,
) -> Result<SessionDigest> {
    if !no_cache {
        if let Some(cached) = cache::load_cached_digest(session) {
            if !use_llm || cached.llm_enhanced {
                return Ok(cached);
            }
        }
    }

    let mut digest = extractor::extract_digest(session, messages);

    if use_llm {
        match llm::enhance_digest(session, messages, &digest) {
            Ok(enhanced) => digest = enhanced,
            Err(e) => {
                eprintln!("Warning: LLM enhancement failed ({e}), using rule-based digest");
            }
        }
    }

    if let Err(e) = cache::save_digest(session, &digest) {
        eprintln!("Warning: failed to cache digest ({e})");
    }

    Ok(digest)
}

pub fn format_digest(digest: &SessionDigest) -> String {
    let mut out = String::new();

    out.push_str(&format!("# Session Digest: {}\n", truncate(&digest.intent, 80)));
    out.push_str(&format!(
        "# Provider: {} | Project: {} | {}\n",
        digest.provider, digest.project_name, digest.time_range,
    ));
    if digest.llm_enhanced {
        out.push_str("# (LLM enhanced)\n");
    }
    out.push('\n');

    out.push_str("## Intent\n");
    out.push_str(&digest.intent);
    out.push_str("\n\n");

    if !digest.key_decisions.is_empty() {
        out.push_str("## Key Decisions\n");
        for d in &digest.key_decisions {
            out.push_str(&format!("- {}\n", d.description));
            if let Some(r) = &d.rationale {
                out.push_str(&format!("  Rationale: {r}\n"));
            }
        }
        out.push('\n');
    }

    if !digest.code_changes.is_empty() {
        out.push_str("## Code Changes\n");
        for c in &digest.code_changes {
            let desc = c
                .description
                .as_deref()
                .map(|d| format!(": {d}"))
                .unwrap_or_default();
            out.push_str(&format!("- {} {}{}\n", c.action, c.file_path, desc));
        }
        out.push('\n');
    }

    if !digest.remaining_issues.is_empty() {
        out.push_str("## Remaining Issues\n");
        for issue in &digest.remaining_issues {
            out.push_str(&format!("- {issue}\n"));
        }
        out.push('\n');
    }

    out.push_str("## Conclusion\n");
    out.push_str(&digest.conclusion);
    out.push('\n');

    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let boundary = s.char_indices()
            .take_while(|(i, _)| *i < max)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(max);
        format!("{}...", &s[..boundary])
    }
}
