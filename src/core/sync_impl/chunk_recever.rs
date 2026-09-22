use crate::core::{
    TinyResult,
    sync_impl::tool_executor::ToolCallResult,
    types::{ToolCall, UserMessage},
};

#[derive(Debug)]
pub enum Chunk {
    /// assistant message content from llm
    Content(String),
    /// assistant reasoning content from llm
    ReasoningContent(String),
    /// assistant tool call from llm
    ToolCalls(ToolCall),
    /// tool call result from tool executor
    ToolResult(ToolCallResult),
    /// user input message
    UserMessage(UserMessage),
}

pub trait ChunkReceiver {
    fn receive(&self, chunk: Chunk) -> TinyResult<()>;
}
