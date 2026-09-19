use serde::{Deserialize, Serialize};

// These structures will follow the openai requirment to use serde_json to do deserialize

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct OpenAIToolFunction {
    pub name: String,
    pub arguments: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct OpenAIToolCall {
    pub id: String,
    pub index: u32,
    pub r#type: String,
    pub function: OpenAIToolFunction,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct OpenaiChoiceDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub reasoning: Option<String>,
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct OpenAIChoice {
    pub index: u32,
    pub delta: OpenaiChoiceDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct OpenAIChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub system_fingerprint: String,
    pub choices: Vec<OpenAIChoice>,
}
