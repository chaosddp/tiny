pub mod agent;
pub mod chat;
pub mod decision;
pub mod error;

pub use agent::{ChatOptions, EventHandler, base_loop};
pub use chat::chunk::Chunk;
pub use chat::messages::{
    AssistantMessage, ChatMessage, FinishReason, ImageDetail, TokenUsage, ToolCall, ToolMessage,
    UserMessage,
};
pub use chat::tool::Tool;
pub type TinyResult<T> = Result<T, error::Error>;
pub type TinyError = error::Error;
