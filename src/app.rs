use std::{
    collections::HashMap,
    env::{current_dir, home_dir},
    fs::{File, OpenOptions},
    io::{BufReader, Read, Write},
    sync::Arc,
};

use log::debug;
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    config::Configurations,
    lua::{register_all, thread::registry::LuaChannelRegistry},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatOptions {
    pub image: Option<String>,
    pub allow: bool,
    pub no_memory: bool,
    pub model: Option<String>,
    pub session: Option<String>,
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self {
            image: Default::default(),
            allow: false,
            no_memory: false,
            model: Default::default(),
            session: Default::default(),
        }
    }
}

impl IntoLua for &ChatOptions {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let t = lua.create_table()?;

        t.set("image", self.image.clone())?;
        t.set("allow", self.allow)?;
        t.set("no_memory", self.no_memory)?;
        t.set("model", self.model.clone())?;
        t.set("session", self.session.clone())?;

        Ok(LuaValue::Table(t))
    }
}

pub struct TinyApplication {
    // main thread for tiny
    lua: Lua,
    config: Option<Configurations>,
    channel_registry: Arc<LuaChannelRegistry>,
}

impl TinyApplication {
    pub fn new() -> LuaResult<Self> {
        let lua = Lua::new();

        let mut inst = Self {
            lua,
            config: None,
            channel_registry: Arc::new(LuaChannelRegistry::default()),
        };

        inst.initialize()?;

        Ok(inst)
    }

    // load configuration and call in
    pub fn load(&self) -> LuaResult<()> {
        let mut loaded = false;

        // if there is a main.lua under '.tiny' folder we use it as entry script
        if let Ok(working_dir) = current_dir() {
            let workspace_dir = working_dir.join(".tiny");

            if !workspace_dir.exists() {
                std::fs::create_dir_all(&workspace_dir)?;
            }

            let entry_script = workspace_dir.join("main.lua");

            if entry_script.exists() && entry_script.is_file() {
                let mut script_content = String::new();
                let mut file = File::open(entry_script)?;

                file.read_to_string(&mut script_content)?;

                self.lua.load(script_content).exec()?;

                loaded = true;
            }
        }

        let globals = self.lua.globals();

        if !loaded {
            let require: LuaFunction = globals.get("require")?;

            require.call::<()>("main")?;
        }

        let conf_func: LuaFunction = globals.get_path("tiny.conf")?;

        // call tiny.conf
        if let Some(config) = &self.config {
            conf_func.call::<()>(config)?;
        } else {
            conf_func.call::<()>(self.lua.create_table()?)?;
        }

        // call tiny.load
        let load_func: LuaFunction = globals.get_path("tiny.load")?;

        load_func.call::<()>(())?;

        Ok(())
    }

    pub fn chat(&self, message: &str, options: &ChatOptions) -> LuaResult<()> {
        // called in chat command

        let chat_func: LuaFunction = self.lua.globals().get_path("tiny.chat")?;

        chat_func.call::<()>((message.to_string(), options))?;

        Ok(())
    }

    pub fn session(&self, command: &str, options: impl IntoLua) -> LuaResult<()> {
        // called in session command
        let session_func: LuaFunction = self.lua.globals().get_path("tiny.session")?;

        session_func.call::<()>((command.to_string(), options))?;

        Ok(())
    }

    pub fn script(&self, script_file: &str, args: Vec<String>) -> LuaResult<()> {
        let mut script_content = String::new();

        {
            let mut file = File::open(script_file)?;
            file.read_to_string(&mut script_content)?;
        }

        self.lua.globals().set("arg", args)?;
        self.lua.load(script_content).exec()?;

        Ok(())
    }

    fn initialize(&mut self) -> LuaResult<()> {
        // initialize home dir with configurations
        let Some(home) = home_dir() else {
            return Err(LuaError::RuntimeError("Cannot find HOME dir".into()));
        };

        let tiny_home_root = home.join(".tiny");

        if !tiny_home_root.exists() {
            std::fs::create_dir_all(&tiny_home_root)?;
        }

        // default json
        let default_config_file = tiny_home_root.join("config.json");

        if default_config_file.is_dir() {
            return Err(LuaError::RuntimeError("config.json is a directory".into()));
        }

        // prepare configuration files
        let mut config = if !default_config_file.exists() {
            // create a new config.json with default configurations

            let config = Configurations::default();

            let config_json_str = serde_json::to_string_pretty(&config).map_err(|e| {
                LuaError::RuntimeError(format!("fail to dump json: {}", e.to_string()))
            })?;

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(default_config_file)?;

            file.write(&config_json_str.into_bytes())?;

            config
        } else {
            let file = File::open(default_config_file)?;

            serde_json::from_reader(BufReader::new(file)).map_err(|e| {
                LuaError::RuntimeError(format!(
                    "fail to parse configuration file: {}",
                    e.to_string()
                ))
            })?
        };

        // config file under current folder?
        if let Ok(working_dir) = current_dir() {
            let workspace_dir = working_dir.join(".tiny");

            if !workspace_dir.exists() {
                std::fs::create_dir_all(&workspace_dir)?;
            }

            let workspace_config_file = workspace_dir.join(".tiny.json");

            if workspace_config_file.is_file() && workspace_config_file.exists() {
                let mut workspace_config_str = String::new();
                let mut file = File::open(workspace_config_file)?;

                file.read_to_string(&mut workspace_config_str)?;

                if let Ok(workspace_config) =
                    serde_json::from_str::<Configurations>(&workspace_config_str)
                {
                    // do merge
                    if let Some(default_model) = workspace_config.default_model {
                        config.default_model = Some(default_model);
                    }

                    if let Some(tools) = workspace_config.tools {
                        if let Some(default_tools) = &mut config.tools {
                            for tool in tools {
                                if !default_tools.contains(&tool) {
                                    default_tools.push(tool);
                                }
                            }
                        } else {
                            config.tools = Some(tools);
                        }
                    }

                    if let Some(extensions) = workspace_config.extensions {
                        if let Some(default_extensions) = &mut config.extensions {
                            for ext in extensions {
                                if !default_extensions.contains(&ext) {
                                    default_extensions.push(ext);
                                }
                            }
                        } else {
                            config.extensions = Some(extensions);
                        }
                    }

                    //
                    if let Some(models) = workspace_config.models {
                        if let Some(default_models) = &mut config.models {
                            let keys: HashMap<String, bool> = default_models
                                .iter()
                                .map(|m| (m.name.to_string(), true))
                                .collect();

                            for model in models {
                                if !keys.contains_key(&model.name) {
                                    default_models.push(model);
                                }
                            }
                        } else {
                            config.models = Some(models);
                        }
                    }
                }
            }
        }

        debug!(
            "initialized with configuration: \n{}",
            serde_json::to_string_pretty(&config).unwrap()
        );

        self.config = Some(config);

        // create global table 'tiny'
        let globals = self.lua.globals();

        let tiny_table: LuaTable = self.lua.load(include_str!("tiny.lua")).eval()?;

        globals.set("tiny", tiny_table)?;

        // register utilities
        register_all(&self.lua)?;

        // register thread here as we need to keep the channel registry
        crate::lua::thread::thread::register(&self.lua, self.channel_registry.clone())?;

        // update lua package search path to executable folder by default
        let exec_path = std::env::current_exe().unwrap();
        let exec_folder = exec_path.parent().unwrap();

        let package_table = globals.get::<LuaTable>("package")?;
        package_table.set(
            "path",
            format!(
                "{0}/tiny/?.lua;{0}/tiny/?/init.lua;",
                exec_folder.to_string_lossy()
            ),
        )?;

        Ok(())
    }
}
