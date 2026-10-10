use std::{fs::File, io::Read, vec};

use log::debug;
use mlua::prelude::*;

use crate::{
    agent::{
        bridges::{
            chat::LuaChatClient, extensions::load_extension, tools::executor::LuaToolExecutor,
        },
        config::Configurations,
    },
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
    configs: Configurations,

    messages: Vec<ChatMessage>,

    chat_client: LuaChatClient,
    tool_executor: LuaToolExecutor,
    context: LoopContext,
}

impl TinyAgent {
    pub fn new() -> TinyResult<Self> {
        let lua = Lua::new();
        let globals = lua.globals();

        let exec_path = std::env::current_exe().unwrap();
        let mut exec_folder = exec_path.parent().unwrap();

        #[cfg(debug_assertions)]
        {
            exec_folder = exec_folder // debug
                .parent() // target
                .unwrap()
                .parent() // workspace
                .unwrap();
        }
        // update lua package search path
        let package_table = globals.get::<LuaTable>("package")?;
        package_table.set(
            "path",
            format!(
                "{0}/tiny/core/?.lua;{0}/tiny/core/?/init.lua;{0}/tiny/extensions/?.lua;{0}/tiny/extensions/?/init.lua",
                exec_folder.to_string_lossy()
            ),
        )?;

        register_all(&lua)?;

        // add tiny table
        let tiny_table = lua.create_table()?;

        // chat_clients table
        let chat_clients_table = lua.create_table()?;

        tiny_table.set("chat_clients", chat_clients_table)?;

        globals.set("tiny", tiny_table)?;

        let mut main_script_str = String::new();

        {
            let main_lua_path = exec_folder.join("tiny").join("core").join("init.lua");

            let mut file = File::open(main_lua_path)?;

            file.read_to_string(&mut main_script_str)?;
        }

        lua.load(main_script_str).exec()?;

        // load configurations (.tiny.json)
        let mut config_str = String::new();

        {
            let mut file = File::open(".tiny.json")?;

            file.read_to_string(&mut config_str)?;
        }

        let configs: Configurations = serde_json::from_str(&config_str)?;

        // load extensions
        for ext_name in &configs.extensions {
            let ext_path = exec_folder.join("tiny").join("extensions").join(ext_name);

            debug!("loading extension from: {:?}", ext_path);

            if ext_path.exists() && ext_path.is_dir() {
                debug!("loaded extension: {}", ext_name);

                load_extension(&lua, ext_path)?;
            }
        }

        let context = LoopContext {};
        let tool_executor = LuaToolExecutor::new();
        let chat_client = LuaChatClient::new(&lua)?;

        Ok(Self {
            lua,
            messages: vec![ChatMessage::System("You are a helpfule assistant".into())],
            context,
            tool_executor,
            chat_client,
            configs,
        })
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
        self.messages.push(ChatMessage::User(message));

        let tools = vec![];

        base_loop(
            &mut self.messages,
            &options,
            |messages, tools, options, receiver| {
                self.chat_client
                    .chat(client, messages, Some(tools), options, Some(receiver))
            },
            &tools,
            |tool, param| Ok(self.tool_executor.execute(&tool, param)?),
            chat_chunk_cb,
            &self.context,
        )?;

        Ok(())
    }
}
