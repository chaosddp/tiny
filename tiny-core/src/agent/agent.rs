use mlua::prelude::*;

use crate::{
    agent::{Configurations, LuaChatClient, LuaEventHandlers, LuaToolExecutor},
    core::{ChatMessage, ChatOptions, Chunk, TinyResult, Tool, UserMessage, base_loop},
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
