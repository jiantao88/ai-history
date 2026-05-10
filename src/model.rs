use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::System => write!(f, "system"),
            Role::Tool => write!(f, "tool"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub provider: String,
    pub name: String,
    pub path: String,
    pub session_count: usize,
    pub last_modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionMetadata {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files_touched: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools_used: Vec<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub has_errors: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub provider: String,
    pub id: String,
    pub file_path: String,
    pub project_name: String,
    pub message_count: usize,
    pub first_time: String,
    pub last_time: String,
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SessionMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub timestamp: String,
    pub text: String,
    pub tool_name: Option<String>,
    pub tool_input: Option<String>,
    pub tool_output: Option<String>,
    pub model: Option<String>,
    pub thinking: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub message: Message,
    pub session_id: String,
    pub project_name: String,
    pub provider: String,
    pub score: f64,
    pub match_index: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub context_before: Vec<Message>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub context_after: Vec<Message>,
}
