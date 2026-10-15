use std::{fs::File, io::Read, path::PathBuf, vec};

use mlua::prelude::*;

use crate::{
    agent::{
        Configurations, LuaChatClient, LuaEventHandlers, LuaToolExecutor, TinyAgent,
        session::manager::SessionManager,
    },
    core::{ChatMessage, TinyError, TinyResult},
    lua::register_all,
};

/// build for TinyAgent
pub struct TinyAgentBuilder {
    lua: Lua,
    configs: Option<Configurations>,
    tool_executor: Option<LuaToolExecutor>,
    prompt: String,
}

impl TinyAgentBuilder {
    /// create a builder with an initialized lua vm, it will do following
    ///
    /// 1. set the package path
    /// 2. register all the lua utilities
    /// 3. create a global table named as 'tiny'
    /// 4. create a table in 'tiny' named as 'chat_clients' that used to collect different chat provider
    /// 5. create a table in 'tiny' named as 'event_handlers' that used to collect event handler objects with event callbacks
    /// 6. load the entry script 'tiny/main.lua' to start
    pub fn new(root: &PathBuf) -> TinyResult<Self> {
        let lua = Lua::new();

        let globals = lua.globals();
        let require = globals.get::<LuaFunction>("require")?;

        // update lua package search path
        let package_table = globals.get::<LuaTable>("package")?;
        package_table.set(
            "path",
            format!(
                "{0}/tiny/?.lua;{0}/tiny/?/init.lua;{0}/extensions/?.lua;{0}/extensions/?/init.lua",
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

        // set the tiny table to global
        globals.set("tiny", tiny_table)?;

        // require the main.lua as our entry script
        require.call::<()>("main")?;

        Ok(Self {
            lua,
            configs: None,
            tool_executor: None,
            prompt: "You are a helpful assistant".into(),
        })
    }

    /// change the default prompt
    pub fn prompt(self, prompt: &str) -> Self {
        Self {
            prompt: prompt.to_string(),
            ..self
        }
    }

    /// load configurations, by default it should '.tiny.json' under current folder
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

    /// load extensions according to configurations from same path of the 'tiny' package
    pub fn load_exetensions(self) -> TinyResult<Self> {
        let Some(configs) = &self.configs else {
            return Err(TinyError::RuntimeError(
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

    // load tools according to configurations from specified path 'tools_root', it will also load default lua utilities under 'tiny/core'
    pub fn load_tools(self, core_root: PathBuf, tools_root: PathBuf) -> TinyResult<Self> {
        let Some(configs) = &self.configs else {
            return Err(TinyError::RuntimeError(
                "load configuration first to determine which extensions to active.".into(),
            ));
        };

        let tool_executor = LuaToolExecutor::from_tools(&core_root, &tools_root, &configs.tools)?;

        Ok(Self {
            tool_executor: Some(tool_executor),
            ..self
        })
    }

    pub fn build(self) -> TinyResult<TinyAgent> {
        let session_manager_backend = self
            .lua
            .globals()
            .get_path::<LuaTable>("tiny.session_manager")?;

        let chat_client = LuaChatClient::new(&self.lua)?;
        let messages = vec![ChatMessage::System(self.prompt)];
        let configs = self
            .configs
            .ok_or(TinyError::RuntimeError("no configuration loaded".into()))?;
        let tool_executor = self.tool_executor.ok_or(TinyError::RuntimeError(
            "tool executor not initialized".into(),
        ))?;
        let tools = tool_executor.get_tools();
        let handlers = LuaEventHandlers::new(
            self.lua
                .globals()
                .get_path::<LuaTable>("tiny.event_handlers")?,
        );

        Ok(TinyAgent {
            lua: self.lua,
            configs,
            messages,
            chat_client,
            tool_executor,
            handlers,
            tools,
            session_manager: SessionManager::new(session_manager_backend),
        })
    }
}
