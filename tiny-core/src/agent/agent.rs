use std::{fs::File, io::Read};

use log::debug;
use mlua::prelude::*;

use crate::{
    agent::{
        Configurations, LuaChatClient, LuaEventHandlers, LuaToolExecutor,
        session::manager::SessionManager,
    },
    core::{
        ChatMessage, ChatOptions, Chunk, EventHandler, TinyError, TinyResult, Tool, UserMessage,
        base_loop,
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
    pub(crate) session_manager: SessionManager,
    pub(crate) tools: Vec<Tool>,
}

impl TinyAgent {
    fn init_session(&mut self) -> TinyResult<()> {
        match self.session_manager.last_session()? {
            Some(session_name) => {
                self.session_manager.switch_session(&session_name)?;

                let messages = self.session_manager.load_memory(&session_name)?;

                for msg in messages {
                    self.messages.push(msg);
                }
            }
            _ => {
                let new_session_name = self.session_manager.new_session()?;

                // NOTE: we should switch session before emit messages, or current messages will not be stored
                self.session_manager.switch_session(&new_session_name)?;

                // since it is a new session, we need to emit current messages except system message, as it would be changed
                for message in &self.messages {
                    match message {
                        ChatMessage::System(_) => {}
                        _ => {
                            self.handlers.emit("message", &message)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn init(&mut self) -> TinyResult<()> {
        self.init_session()?;

        let system_message = self.messages.get_mut(0).unwrap();

        let ChatMessage::System(prompt) = system_message else {
            return Err(TinyError::RuntimeError(
                "first message should be system prompt.".into(),
            ));
        };

        if let Ok(new_prompt) = self.handlers.collect("prompt", prompt) {
            debug!("prompt changed to: {}", new_prompt);

            *prompt = new_prompt;
        }

        Ok(())
    }

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
        let use_message = ChatMessage::User(message);

        self.handlers.emit("message", &&use_message)?;

        self.messages.push(use_message);

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

    pub fn execute(&self, file: &str, args: Vec<String>) -> TinyResult<()> {
        let mut script = String::new();

        {
            let mut file = File::open(file)?;

            file.read_to_string(&mut script)?;
        }

        self.lua.globals().set("arg", args)?;

        self.lua.load(script).exec()?;

        Ok(())
    }
}
