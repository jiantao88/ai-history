use crate::model::{Message, Role, Session};

pub fn export_session(session: &Session, messages: &[Message]) -> String {
    let mut out = String::new();

    let summary = session.summary.as_deref().unwrap_or(&session.id);
    out.push_str(&format!("# Session: {summary}\n\n"));
    out.push_str(&format!(
        "**Project**: {} | **Provider**: {} | **Date**: {}\n\n",
        session.project_name, session.provider, session.first_time,
    ));
    out.push_str("---\n\n");

    for msg in messages {
        let heading = match msg.role {
            Role::User => "## User",
            Role::Assistant => "## Assistant",
            Role::System => "## System",
            Role::Tool => "### Tool",
        };

        out.push_str(heading);
        out.push('\n');

        if let Some(ref thinking) = msg.thinking {
            out.push_str("\n<details><summary>Thinking</summary>\n\n");
            out.push_str(thinking);
            out.push_str("\n\n</details>\n");
        }

        if let Some(ref tool) = msg.tool_name {
            out.push_str(&format!("\n**Tool**: {tool}\n"));
            if let Some(ref input) = msg.tool_input {
                out.push_str(&format!("\n```\n{input}\n```\n"));
            }
            if let Some(ref output) = msg.tool_output {
                let preview = if output.len() > 2000 {
                    format!("{}...(truncated)", &output[..2000])
                } else {
                    output.clone()
                };
                out.push_str(&format!("\n> {}\n", preview.replace('\n', "\n> ")));
            }
        }

        if !msg.text.is_empty() {
            out.push('\n');
            out.push_str(&msg.text);
            out.push('\n');
        }

        out.push('\n');
    }

    out
}
