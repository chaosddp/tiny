use crate::core::{
    TinyResult, sync_impl::chunk_recever::ChunkReceiver, types::{AssistantMessage, Message},
};

/// Reasoning depth. It is applicable to reasoning models, User [`ReasoningEffort::Other`] for customize depth.
#[derive(Debug, Clone)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
    Other(String),
}

/// Thinking type
#[derive(Debug, Clone)]
pub enum ThinkingType {
    /// for enabling
    Enabled,
    /// for disabling
    Disabled,
    /// for adaptive mode
    Adaptive,
    Other(String),
}

/// Thinking mode settings
#[derive(Debug, Clone)]
pub struct ThinkingOptions {
    pub t_type: ThinkingType,
    /// The maximum number of tokens for the reasoning process, default is 8192
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

/// Options for chat interface
#[derive(Debug, Clone)]
pub struct ChatOptions {
    pub model: String,
    /// Base of the provider api interface
    pub base_url: String,
    /// Api key to access the api
    pub api_key: String,
    /// Whether to enable streaming output (SSE)
    pub stream: bool,
    /// The maximum number of tokens that can be generated in a single response
    pub max_token: u32,
    /// reasoning depth
    pub reasoning_effort: Option<ReasoningEffort>,
    /// thinking options
    pub thinking: Option<ThinkingOptions>,
    /// Whether the last chunk in streaming mode contains token usage, for stream=true only.
    pub include_usage: bool,
}

pub trait ChatClient {
    fn chat(
        &self,
        options: &ChatOptions,
        messages: &Vec<Message>,                 // current messages
        chunk_receiver: &Box<dyn ChunkReceiver>, // sender that used to send SSE chunk
    ) -> TinyResult<AssistantMessage>;
}
