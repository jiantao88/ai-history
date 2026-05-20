use crate::model::{Message, SearchResult, Session};
use crate::provider::ProviderRegistry;
use crate::scoring::Bm25Scorer;
use anyhow::Result;

pub struct SearchOptions {
    pub query: String,
    pub limit: usize,
    pub context_window: usize,
    pub require_all_terms: bool,
    pub sort_by_time: bool,
}

pub fn search_all(
    registry: &ProviderRegistry,
    opts: &SearchOptions,
    provider_filter: Option<&[String]>,
) -> Result<Vec<SearchResult>> {
    registry.search_all(opts, provider_filter)
}

struct SessionMessages {
    session: Session,
    messages: Vec<Message>,
    provider_id: String,
}

/// Shared BM25 search engine used by all providers.
/// Collects all messages, scores with BM25, ranks, attaches context.
pub fn rank_and_search(
    entries: Vec<(Session, Vec<Message>, String)>,
    opts: &SearchOptions,
) -> Vec<SearchResult> {
    // Collect corpus: all message texts for BM25 statistics
    let corpus_texts: Vec<String> = entries
        .iter()
        .flat_map(|(_, msgs, _)| msgs.iter().map(|m| m.text.clone()))
        .collect();
    let corpus_refs: Vec<&str> = corpus_texts.iter().map(|s| s.as_str()).collect();

    let scorer = Bm25Scorer::new(&opts.query, &corpus_refs);

    let sessions: Vec<SessionMessages> = entries
        .into_iter()
        .map(|(session, messages, provider_id)| SessionMessages {
            session,
            messages,
            provider_id,
        })
        .collect();

    // Score and filter all messages
    let mut candidates: Vec<SearchResult> = Vec::new();

    for sm in &sessions {
        for (idx, msg) in sm.messages.iter().enumerate() {
            if !scorer.matches(&msg.text, opts.require_all_terms) {
                continue;
            }

            let score = scorer.score(&msg.text);

            candidates.push(SearchResult {
                message: msg.clone(),
                session_id: sm.session.id.clone(),
                project_name: sm.session.project_name.clone(),
                provider: sm.provider_id.clone(),
                score,
                match_index: idx,
                context_before: Vec::new(),
                context_after: Vec::new(),
            });
        }
    }

    // Sort
    if opts.sort_by_time {
        candidates.sort_by(|a, b| b.message.timestamp.cmp(&a.message.timestamp));
    } else {
        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    // Truncate to limit
    candidates.truncate(opts.limit);

    // Attach context if requested
    if opts.context_window > 0 {
        // Build a lookup from (provider, session_id) -> messages
        let session_map: std::collections::HashMap<(&str, &str), &[Message]> = sessions
            .iter()
            .map(|sm| {
                (
                    (sm.provider_id.as_str(), sm.session.id.as_str()),
                    sm.messages.as_slice(),
                )
            })
            .collect();

        for result in &mut candidates {
            let key = (result.provider.as_str(), result.session_id.as_str());
            if let Some(messages) = session_map.get(&key) {
                attach_context(result, messages, result.match_index, opts.context_window);
            }
        }
    }

    candidates
}

fn attach_context(
    result: &mut SearchResult,
    all_messages: &[Message],
    match_idx: usize,
    window: usize,
) {
    let start = match_idx.saturating_sub(window);
    let end = (match_idx + 1 + window).min(all_messages.len());

    if start < match_idx {
        result.context_before = all_messages[start..match_idx].to_vec();
    }
    if match_idx + 1 < end {
        result.context_after = all_messages[match_idx + 1..end].to_vec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Role;

    fn make_msg(text: &str) -> Message {
        Message {
            role: Role::User,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            text: text.to_string(),
            tool_name: None,
            tool_input: None,
            tool_output: None,
            model: None,
            thinking: None,
        }
    }

    fn make_session(id: &str) -> Session {
        Session {
            provider: "test".to_string(),
            id: id.to_string(),
            file_path: String::new(),
            project_name: "test-project".to_string(),
            message_count: 0,
            first_time: String::new(),
            last_time: String::new(),
            summary: None,
            metadata: None,
            is_subagent: false,
            parent_session_id: None,
            agent_type: None,
            agent_description: None,
        }
    }

    #[test]
    fn test_context_middle() {
        let messages = vec![
            make_msg("msg0"),
            make_msg("msg1"),
            make_msg("msg2 target"),
            make_msg("msg3"),
            make_msg("msg4"),
        ];

        let mut result = SearchResult {
            message: messages[2].clone(),
            session_id: "s1".to_string(),
            project_name: "p1".to_string(),
            provider: "test".to_string(),
            score: 1.0,
            match_index: 2,
            context_before: Vec::new(),
            context_after: Vec::new(),
        };

        attach_context(&mut result, &messages, 2, 2);
        assert_eq!(result.context_before.len(), 2);
        assert_eq!(result.context_after.len(), 2);
        assert_eq!(result.context_before[0].text, "msg0");
        assert_eq!(result.context_before[1].text, "msg1");
        assert_eq!(result.context_after[0].text, "msg3");
        assert_eq!(result.context_after[1].text, "msg4");
    }

    #[test]
    fn test_context_start() {
        let messages = vec![
            make_msg("msg0 target"),
            make_msg("msg1"),
            make_msg("msg2"),
        ];

        let mut result = SearchResult {
            message: messages[0].clone(),
            session_id: "s1".to_string(),
            project_name: "p1".to_string(),
            provider: "test".to_string(),
            score: 1.0,
            match_index: 0,
            context_before: Vec::new(),
            context_after: Vec::new(),
        };

        attach_context(&mut result, &messages, 0, 2);
        assert_eq!(result.context_before.len(), 0);
        assert_eq!(result.context_after.len(), 2);
    }

    #[test]
    fn test_context_end() {
        let messages = vec![
            make_msg("msg0"),
            make_msg("msg1"),
            make_msg("msg2 target"),
        ];

        let mut result = SearchResult {
            message: messages[2].clone(),
            session_id: "s1".to_string(),
            project_name: "p1".to_string(),
            provider: "test".to_string(),
            score: 1.0,
            match_index: 2,
            context_before: Vec::new(),
            context_after: Vec::new(),
        };

        attach_context(&mut result, &messages, 2, 2);
        assert_eq!(result.context_before.len(), 2);
        assert_eq!(result.context_after.len(), 0);
    }

    #[test]
    fn test_context_zero_window() {
        let messages = vec![make_msg("msg0"), make_msg("msg1 target"), make_msg("msg2")];

        let mut result = SearchResult {
            message: messages[1].clone(),
            session_id: "s1".to_string(),
            project_name: "p1".to_string(),
            provider: "test".to_string(),
            score: 1.0,
            match_index: 1,
            context_before: Vec::new(),
            context_after: Vec::new(),
        };

        attach_context(&mut result, &messages, 1, 0);
        assert_eq!(result.context_before.len(), 0);
        assert_eq!(result.context_after.len(), 0);
    }

    #[test]
    fn test_rank_and_search_scoring() {
        let session = make_session("s1");
        let messages = vec![
            make_msg("the auth middleware has a bug in token validation"),
            make_msg("please fix the CSS styling on the homepage"),
            make_msg("auth token refresh is failing with errors"),
        ];

        let opts = SearchOptions {
            query: "auth bug".to_string(),
            limit: 10,
            context_window: 0,
            require_all_terms: false,
            sort_by_time: false,
        };

        let results = rank_and_search(
            vec![(session, messages, "test".to_string())],
            &opts,
        );

        // Should have 2 results (doc0 and doc2 match "auth")
        assert!(results.len() >= 2);
        // doc0 should rank first (has both "auth" and "bug")
        assert!(results[0].score > results[1].score);
        assert!(results[0].message.text.contains("auth middleware"));
    }

    #[test]
    fn test_rank_and_search_and_mode() {
        let session = make_session("s1");
        let messages = vec![
            make_msg("auth and bug together"),
            make_msg("auth only here"),
            make_msg("bug only here"),
            make_msg("nothing relevant"),
        ];

        let opts = SearchOptions {
            query: "auth bug".to_string(),
            limit: 10,
            context_window: 0,
            require_all_terms: true,
            sort_by_time: false,
        };

        let results = rank_and_search(
            vec![(session, messages, "test".to_string())],
            &opts,
        );

        // Only doc0 matches both terms in AND mode
        assert_eq!(results.len(), 1);
        assert!(results[0].message.text.contains("auth and bug"));
    }
}
