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

    chat_client: LuaChatClient,
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
            let cur_path = std::env::current_exe()
                .unwrap()
                .parent() // debug
                .unwrap()
                .parent() // target
                .unwrap()
                .parent() // workspace
                .unwrap()
                .to_string_lossy()
                .to_string();

            // update lua package search path
            let package_table = globals.get::<LuaTable>("package")?;
            package_table.set(
                "path",
                format!("{0}/tiny/lua/?.lua;{0}/tiny/lua/?/init.lua", cur_path),
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
            let exec_path = std::env::current_exe().unwrap();
            let mut exec_folder = exec_path.parent().unwrap();

            #[cfg(debug_assertions)]
            {
                // to the workspace root, to avoid copy lua files to target folder
                exec_folder = exec_folder.parent().unwrap().parent().unwrap();
            }

            let main_lua_path = exec_folder.join("tiny").join("lua").join("main.lua");

            let mut file = File::open(main_lua_path)?;

            file.read_to_string(&mut main_script_str)?;
        }

        lua.load(main_script_str).exec()?;

        let context = LoopContext {};
        let tool_executor = LuaToolExecutor::new();
        let chat_client = LuaChatClient::new(&lua)?;

        Ok(Self {
            lua,
            messages: vec![ChatMessage::System("You are a helpfule assistant".into())],
            context,
            tool_executor,
            chat_client,
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
