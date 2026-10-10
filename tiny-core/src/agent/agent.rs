use std::{fs::File, io::Read, vec};

use mlua::prelude::*;

use crate::{
    agent::bridges::{chat::LuaChatClient, tools::executor::LuaToolExecutor},
    core::{
        TinyResult,
        agent::{ChatOptions, LoopContext, base_loop},
        chat::{
            chunk::Chunk,
            messages::{ChatMessage, UserMessage},
        },
    },
    lua::register_all,
};

#[allow(dead_code)]
pub struct TinyAgent {
    lua: Lua,

    messages: Vec<ChatMessage>,
    tool_executor: LuaToolExecutor,
    context: LoopContext,
}

impl TinyAgent {
    pub fn new() -> TinyResult<Self> {
        let lua = Lua::new();
        let globals = lua.globals();

        #[cfg(debug_assertions)]
        {
            // setup search path under debug mode, so we do not need to copy lua files to target folder
            let cur_path = std::env::current_dir()
                .map_err(|e| {
                    mlua::Error::RuntimeError(format!(
                        "Fail to get executable path: {}",
                        e.to_string()
                    ))
                })?
                .to_string_lossy()
                .to_string();

            // update lua package search path
            let package_table = globals.get::<LuaTable>("package")?;
            package_table.set(
                "path",
                format!("{0}/lua/?.lua;{0}/lua/?/init.lua", cur_path),
            )?;
        }

        register_all(&lua)?;

        // add tiny table
        let tiny_table = lua.create_table()?;

        // chat_clients table
        let chat_clients_table = lua.create_table()?;

        tiny_table.set("chat_clients", chat_clients_table)?;

        globals.set("tiny", tiny_table)?;

        let mut main_script_str = String::new();

        {
            let mut file = File::open("lua/main.lua")?;

            file.read_to_string(&mut main_script_str)?;
        }

        lua.load(main_script_str).exec()?;

        let context = LoopContext {};
        let tool_executor = LuaToolExecutor::new();

        Ok(Self {
            lua,
            messages: vec![ChatMessage::System("You are a helpfule assistant".into())],
            context,
            tool_executor,
        })
    }

    pub fn chat<F>(
        &mut self,
        client: &str,
        chat_chunk_cb: F,
        message: UserMessage,
    ) -> TinyResult<()>
    where
        F: Fn(Chunk) -> TinyResult<()>,
    {
        let globals = self.lua.globals();

        // TODO: cache the client
        let chat_client_provider_path = format!("tiny.chat_clients.{}", client);
        let chat_client_provider: LuaTable =
            globals.get_path(chat_client_provider_path.as_ref())?;

        let chat_client = LuaChatClient::from_lua_table(&self.lua, chat_client_provider)?;

        self.messages.push(ChatMessage::User(message));

        let options = ChatOptions {
            model: "qwen3.5".into(),
            base_url: "http://localhost:11434/v1".into(),
            api_key: "ollama".into(),
            stream: Some(true),
            stream_include_usage: None,
            max_tokens: Some(64000),
            reasoning_effort: None,
        };


        let tools = vec![];

        base_loop(
            &mut self.messages,
            &options,
            |messages, tools, options, receiver| {
                chat_client.chat(messages, Some(tools), options, Some(receiver))
            },
            &tools,
            |tool, param| Ok(self.tool_executor.execute(&tool, param)?),
            chat_chunk_cb,
            &self.context,
        )?;

        Ok(())
    }
}
