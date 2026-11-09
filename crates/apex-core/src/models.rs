use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingDepth {
    Fast,
    #[default]
    Standard,
    Deep,
}

impl ThinkingDepth {
    pub fn num_predict(&self) -> u32 {
        match self {
            Self::Fast => 1536,
            Self::Standard => 3072,
            Self::Deep => 6144,
        }
    }

    pub fn temperature(&self) -> f32 {
        match self {
            Self::Fast => 0.5,
            Self::Standard => 0.6,
            Self::Deep => 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStreamRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(default = "default_true")]
    pub web_search: bool,
    #[serde(default)]
    pub think_depth: ThinkingDepth,
    pub temperature: Option<f32>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSource {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub snippet: String,
}
