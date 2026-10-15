use std::{fs::File, io::Read, path::PathBuf, vec};

use mlua::prelude::*;

use crate::{
    agent::{
        agent::TinyAgent,
        bridges::{
            chat::LuaChatClient, handlers::LuaEventHandlers, tools::executor::LuaToolExecutor,
        },
        config::Configurations,
    },
    core::{TinyResult, chat::messages::ChatMessage, error::Error},
    lua::register_all,
};

pub struct TinyAgentBuilder {
    lua: Lua,
    configs: Option<Configurations>,
    tool_executor: Option<LuaToolExecutor>,
    prompt: String,
}

impl TinyAgentBuilder {
    pub fn new(root: &PathBuf) -> TinyResult<Self> {
        let lua = Lua::new();

        let globals = lua.globals();
        let require = globals.get::<LuaFunction>("require")?;

        // update lua package search path
        let package_table = globals.get::<LuaTable>("package")?;
        package_table.set(
            "path",
            format!(
                "{0}/tiny/core/?.lua;{0}/tiny/core/?/init.lua;{0}/extensions/?.lua;{0}/extensions/?/init.lua",
                root.to_string_lossy()
            ),
        )?;

        register_all(&lua)?;

        // add tiny table
        let tiny_table = lua.create_table()?;

        // chat_clients table
        let chat_clients_table = lua.create_table()?;

        tiny_table.set("chat_clients", chat_clients_table)?;

        // event handler
        let event_handler_table = lua.create_table()?;

        tiny_table.set("event_handlers", event_handler_table)?;

        globals.set("tiny", tiny_table)?;

        require.call::<()>("main")?;

        Ok(Self {
            lua,
            configs: None,
            tool_executor: None,
            prompt: "You are a helpful assistant".into(),
        })
    }

    pub fn prompt(self, prompt: &str) -> Self {
        Self {
            prompt: prompt.to_string(),
            ..self
        }
    }

    pub fn config(self, config_file: PathBuf) -> TinyResult<Self> {
        let mut config_str = String::new();

        {
            let mut file =
                File::open(config_file).expect("Fail to load .tiny.json, may be not exist.");

            file.read_to_string(&mut config_str)?;
        }

        let configs: Configurations = serde_json::from_str(&config_str)?;

        Ok(Self {
            configs: Some(configs),
            ..self
        })
    }

    pub fn load_exetensions(self) -> TinyResult<Self> {
        let Some(configs) = &self.configs else {
            return Err(Error::RuntimeError(
                "load configuration first to determine which extensions to active.".into(),
            ));
        };

        // main.lua will defined a 'tiny.load_extensions' function that require a extension list
        let load_exetensions = self
            .lua
            .globals()
            .get_path::<LuaFunction>("tiny.load_extensions")?;

        load_exetensions.call::<()>(configs.extensions.clone())?;

        Ok(self)
    }

    pub fn load_tools(self, root: PathBuf) -> TinyResult<Self> {
        let Some(configs) = &self.configs else {
            return Err(Error::RuntimeError(
                "load configuration first to determine which extensions to active.".into(),
            ));
        };

        let tool_executor = LuaToolExecutor::from_tools(&root.join("tools"), &configs.tools)?;

        Ok(Self {
            tool_executor: Some(tool_executor),
            ..self
        })
    }

    pub fn build(self) -> TinyResult<TinyAgent> {
        let chat_client = LuaChatClient::new(&self.lua)?;
        let messages = vec![ChatMessage::System(self.prompt)];
        let configs = self
            .configs
            .ok_or(Error::RuntimeError("no configuration loaded".into()))?;
        let tool_executor = self
            .tool_executor
            .ok_or(Error::RuntimeError("tool executor not initialized".into()))?;
        let tools = tool_executor.get_tools();
        let handlers = Box::new(LuaEventHandlers::new(
            &self.lua,
            self.lua
                .globals()
                .get_path::<LuaTable>("tiny.event_handlers")?,
        )?);

        Ok(TinyAgent {
            lua: self.lua,
            configs,
            messages,
            chat_client,
            tool_executor,
            handlers,
            tools,
        })
    }
}
