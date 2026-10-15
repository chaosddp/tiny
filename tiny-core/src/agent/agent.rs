use mlua::prelude::*;

use crate::{
    agent::{
        bridges::{
            chat::LuaChatClient, handlers::LuaEventHandlers, tools::executor::LuaToolExecutor,
        },
        config::Configurations,
    },
    core::{
        TinyResult,
        agent::{ChatOptions, base_loop},
        chat::{
            chunk::Chunk,
            messages::{ChatMessage, UserMessage},
            tool::Tool,
        },
    },
};

#[allow(dead_code)]
pub struct TinyAgent {
    pub(crate) lua: Lua,
    pub(crate) configs: Configurations,
    pub(crate) messages: Vec<ChatMessage>,
    pub(crate) chat_client: LuaChatClient,
    pub(crate) tool_executor: LuaToolExecutor,
    pub(crate) handlers: LuaEventHandlers,
    pub(crate) tools: Vec<Tool>,
}

impl TinyAgent {
    pub fn chat<F>(
        &mut self,
        client: &str,
        message: UserMessage,
        options: &ChatOptions,
        chat_chunk_cb: F,
    ) -> TinyResult<()>
    where
        F: Fn(Chunk) -> TinyResult<()>,
    {
        self.messages.push(ChatMessage::User(message));

        base_loop(
            &mut self.messages,
            &options,
            |messages, tools, options, receiver| {
                self.chat_client
                    .chat(client, messages, Some(tools), options, Some(receiver))
            },
            &self.tools,
            |tool, param| Ok(self.tool_executor.execute(&tool, param)?),
            chat_chunk_cb,
            &self.handlers,
        )?;

        Ok(())
    }
}
