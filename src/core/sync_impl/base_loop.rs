use crate::core::{
    TinyResult,
    sync_impl::{
        chat_client::{ChatClient, ChatOptions},
        chunk_recever::ChunkReceiver,
        tool_executor::ToolExecutor,
    },
    types::Message,
};

/// Context that passed to plugins in loop
#[derive(Debug)]
pub struct LoopContext {
    // TODO: fields
}

#[derive(Debug)]
pub struct Plugins {}

pub trait AgentLoop {
    fn run_loop(
        &self,
        messages: &mut Vec<Message>,
        options: &ChatOptions,
        chunk_receiver: &Box<dyn ChunkReceiver>,
        chat_client: &Box<dyn ChatClient>,
        tool_executor: &Box<dyn ToolExecutor>,
        plugins: Option<Plugins>,
    ) -> TinyResult<()>;
}
