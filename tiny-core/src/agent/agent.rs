use std::{fs::File, io::Read};

use mlua::prelude::*;

use crate::{
    agent::bridges::chat_client::LuaChatClient,
    core::{
        TinyResult,
        agent::{ChatClient, ChatOptions},
        chat::messages::{self, ChatMessage},
    },
    lua::register_all,
};

pub struct TinyAgent {
    lua: Lua,
}

impl TinyAgent {
    pub fn new() -> TinyResult<Self> {
        let lua = Lua::new();

        Ok(Self { lua })
    }

    pub fn run(&self) -> TinyResult<()> {
        let globals = self.lua.globals();

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

        register_all(&self.lua)?;

        // add tiny table
        let tiny_table = self.lua.create_table()?;

        globals.set("tiny", tiny_table)?;

        let mut main_script_str = String::new();

        {
            let mut file = File::open("lua/main.lua")?;

            file.read_to_string(&mut main_script_str)?;
        }

        self.lua.load(main_script_str).exec()?;

        let chat_client_provider: LuaTable = globals.get_path("tiny.chat_client")?;

        let chat_provider: LuaTable = chat_client_provider.call_function("new", ())?;

        let chat_client = LuaChatClient::from_lua_table(&self.lua, chat_provider)?;

        let mut messages = vec![
            ChatMessage::System("You are a helpfule assistant".into()),
            ChatMessage::User(messages::UserMessage::Text("who are you?".into())),
        ];

        let options = ChatOptions {
            model: "qwen3.5".into(),
            base_url: "http://localhost:11434/v1".into(),
            api_key: "ollama".into(),
            stream: Some(false),
            stream_include_usage: None,
            max_tokens: Some(64000),
            reasoning_effort: None,
        };

        let msg = chat_client.chat(&mut messages, None, &options, None)?;

        println!("{:?}", msg);

        Ok(())
    }
}
