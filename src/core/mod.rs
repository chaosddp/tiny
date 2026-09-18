#[cfg(feature = "async")]
pub mod async_impl;

#[cfg(feature = "sync")]
pub mod sync_impl;

use thiserror::Error;

#[cfg(feature = "async")]
use tokio::sync::mpsc;

#[derive(Error, Debug)]
pub enum TinyError {
    #[error("runtime error")]
    RuntimeError,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ImageDetail {
    Auto,
    Low,
    High,
    Other(String),
}

impl Default for ImageDetail {
    fn default() -> Self {
        ImageDetail::Auto
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ContentPart {
    Text(String),
    Image { url: String, detail: ImageDetail },
    Video(String),
    File(String),
}

pub type RichContent = Vec<ContentPart>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum UserMessage {
    Text(String),
    Parts(RichContent),
}

#[derive(Debug)]
pub struct ToolCall {
    pub name: String,
    pub id: String,
    pub index: u32,
    pub arguments: Option<String>,
}

#[derive(Debug)]
pub struct ToolCallResult {
    pub name: String,
    pub id: String,
    pub result: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FinishReason {
    ToolCall,
    Other(String),
}

#[derive(Debug)]
pub enum Message {
    SystemMessage(String),
    UserMessage(UserMessage),
    AssistantMessage {
        content: Option<String>,
        reasoning_content: Option<String>,
        reasoning_details: Option<Vec<String>>,
        tool_calls: Option<Vec<ToolCall>>,
        finished_reason: FinishReason,
    },
    ToolMessage {
        content: String,
        tool_call_id: String,
        name: String,
    },
}

#[derive(Debug, Clone)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
    Other(String),
}

#[derive(Debug, Clone)]
pub enum ThinkingType {
    Enabled,
    Disabled,
    Adaptive,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct ThinkingOptions {
    pub t_type: ThinkingType,
    pub budget_tokens: u32,
}

impl Default for ThinkingOptions {
    fn default() -> Self {
        Self {
            t_type: ThinkingType::Enabled,
            budget_tokens: 8192,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatOptions {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub stream: bool,
    pub max_token: u32,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub thinking: Option<ThinkingOptions>,
    pub include_usage: bool,
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self {
            model: std::env::var("TINY_DEFAULT_MODEL").unwrap_or(Default::default()),
            base_url: std::env::var("TINY_DEFAULT_BASE_URL").unwrap_or(Default::default()),
            api_key: std::env::var("TINY_DEFAULT_API_KEY").unwrap_or(Default::default()),
            stream: true,
            include_usage: true,
            max_token: 8000,
            reasoning_effort: None,
            thinking: Some(Default::default()),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum MessageChunk {
    Chunk {
        content: Option<String>,
        reasoning_content: Option<String>,
        tool_calls: Option<Vec<ToolCall>>,
    },
    Error(String),
}

#[derive(Debug)]
pub struct ToolParameter {
    pub name: String,
    pub p_type: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}
