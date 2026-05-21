use anyhow::Result;

use crate::model::{Message, Role, Session};
use super::SessionDigest;

pub fn enhance_digest(
    _session: &Session,
    messages: &[Message],
    rule_digest: &SessionDigest,
) -> Result<SessionDigest> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY not set. Use --llm only with a valid API key."))?;

    let base_url = std::env::var("ANTHROPIC_BASE_URL")
        .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
    let api_url = format!("{}/v1/messages", base_url.trim_end_matches('/'));

    let condensed = build_condensed_conversation(messages);
    let rule_text = super::format_digest(rule_digest);

    let prompt = format!(
        "You are summarizing an AI coding session. Below is the conversation (condensed to user/assistant text only), \
         followed by a preliminary rule-based digest.\n\n\
         ## Conversation\n\n{condensed}\n\n\
         ## Preliminary Digest\n\n{rule_text}\n\n\
         ## Task\n\n\
         Enhance this digest with clearer, more informative narrative. Keep the exact same structure \
         (Intent, Key Decisions, Code Changes, Remaining Issues, Conclusion). \
         Output valid JSON matching this schema:\n\
         {{\"intent\": \"...\", \"key_decisions\": [{{\"description\": \"...\", \"rationale\": \"...\"}}], \
         \"code_changes\": [{{\"file_path\": \"...\", \"action\": \"Created|Modified|Deleted\", \"description\": \"...\"}}], \
         \"remaining_issues\": [\"...\"], \"conclusion\": \"...\"}}\n\n\
         Output ONLY the JSON object, no other text."
    );

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&api_url)
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 2048,
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

    let enhanced: serde_json::Value = serde_json::from_str(&json_text)
        .map_err(|e| anyhow::anyhow!("Failed to parse LLM JSON: {e}"))?;

    let mut digest = rule_digest.clone();
    digest.llm_enhanced = true;

    if let Some(intent) = enhanced["intent"].as_str() {
        digest.intent = intent.to_string();
    }
    if let Some(conclusion) = enhanced["conclusion"].as_str() {
        digest.conclusion = conclusion.to_string();
    }
    if let Some(decisions) = enhanced["key_decisions"].as_array() {
        digest.key_decisions = decisions
            .iter()
            .filter_map(|d| {
                Some(super::Decision {
                    description: d["description"].as_str()?.to_string(),
                    rationale: d["rationale"].as_str().map(String::from),
                })
            })
            .collect();
    }
    if let Some(changes) = enhanced["code_changes"].as_array() {
        digest.code_changes = changes
            .iter()
            .filter_map(|c| {
                Some(super::FileChange {
                    file_path: c["file_path"].as_str()?.to_string(),
                    action: match c["action"].as_str()? {
                        "Created" => super::FileAction::Created,
                        "Deleted" => super::FileAction::Deleted,
                        _ => super::FileAction::Modified,
                    },
                    description: c["description"].as_str().map(String::from),
                })
            })
            .collect();
    }
    if let Some(issues) = enhanced["remaining_issues"].as_array() {
        digest.remaining_issues = issues
            .iter()
            .filter_map(|i| i.as_str().map(String::from))
            .collect();
    }

    Ok(digest)
}

fn build_condensed_conversation(messages: &[Message]) -> String {
    let mut out = String::new();
    let max_chars = 32_000; // ~8K tokens

    for msg in messages {
        if msg.tool_output.is_some() && msg.role == Role::Tool {
            continue;
        }
        if msg.tool_name.is_some() {
            continue;
        }
        match msg.role {
            Role::User => {
                let text = msg.text.trim();
                if !text.is_empty() {
                    out.push_str("User: ");
                    out.push_str(text);
                    out.push_str("\n\n");
                }
            }
            Role::Assistant => {
                let text = msg.text.trim();
                if !text.is_empty() {
                    out.push_str("Assistant: ");
                    out.push_str(text);
                    out.push_str("\n\n");
                }
                if let Some(thinking) = &msg.thinking {
                    let t = thinking.trim();
                    if !t.is_empty() {
                        out.push_str("[Thinking]: ");
                        out.push_str(t);
                        out.push_str("\n\n");
                    }
                }
            }
            _ => {}
        }

        if out.len() > max_chars {
            break;
        }
    }

    if out.len() > max_chars {
        // Keep first and last portions
        let half = max_chars / 2;
        let start = &out[..half];
        let end = &out[out.len() - half..];
        format!("{start}\n\n[... middle of conversation omitted ...]\n\n{end}")
    } else {
        out
    }
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
