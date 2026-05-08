use crate::model::{Message, Role, Session};

pub fn export_session(session: &Session, messages: &[Message]) -> String {
    let mut out = String::new();

    let summary = session.summary.as_deref().unwrap_or(&session.id);
    out.push_str(&format!("# Context from: {summary}\n"));
    out.push_str(&format!(
        "# Provider: {} | Project: {} | Date: {}\n\n",
        session.provider, session.project_name, session.first_time,
    ));

    for msg in messages {
        if msg.tool_name.is_some() || msg.tool_output.is_some() {
            continue;
        }
        match msg.role {
            Role::User => {
                let trimmed = msg.text.trim();
                if !trimmed.is_empty() && !is_tool_result_content(trimmed) {
                    out.push_str("User:\n");
                    out.push_str(trimmed);
                    out.push_str("\n\n");
                }
            }
            Role::Assistant => {
                let trimmed = msg.text.trim();
                if !trimmed.is_empty() {
                    out.push_str("Assistant:\n");
                    out.push_str(trimmed);
                    out.push_str("\n\n");
                }
            }
            _ => {}
        }
    }

    out
}

fn is_tool_result_content(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with("[result]")
        || trimmed.starts_with("/Users/")
        || trimmed.starts_with("C:\\")
        || (trimmed.len() > 500 && !trimmed.contains(' '))
}
