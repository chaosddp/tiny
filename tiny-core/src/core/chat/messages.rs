use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum ImageDetail {
    Auto,
    High,
    Low,
    Other(String),
}
#[derive(Debug, Deserialize, Serialize)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCall,
    ContentFilter,
    Other(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub index: usize,
    pub arguments: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenUsage {
    pub prompt: usize,
    pub completion: usize,
    pub total: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolMessage {
    pub id: String,
    pub name: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AssistantMessage {
    pub content: Option<String>,
    pub reasoning: Option<String>,
    pub reasoning_details: Option<Vec<String>>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub finish_reason: FinishReason,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserImageContentPart {
    pub image: String,
    pub detail: ImageDetail,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum UserContentPart {
    Text(String),
    Video(String),
    File(String),
    Image { image: String, detail: ImageDetail },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum UserMessage {
    Text(String),
    Parts(Vec<UserContentPart>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ChatMessage {
    System(String),
    User(UserMessage),
    Assistant(AssistantMessage),
    Tool(ToolMessage),
}
