use std::{fs::File, io::Read, path::Path};

use crate::{
    TinyResult,
    agent::{
        configuration::Configurations,
        context::{Context, ContextBuilder},
        extensions::load_extension,
    },
    utils::register_all,
};
use log::debug;
use mlua::prelude::*;

pub struct TinyAgent {
    lua: Lua,
    context: Context,
}

impl TinyAgent {
    pub fn new() -> TinyResult<Self> {
        let lua = Lua::new();

        let globals = lua.globals();

        let exec_path = std::env::current_exe().unwrap();
        let mut exec_folder = exec_path.parent().unwrap();

        // for debugging, use folder under workspace
        #[cfg(debug_assertions)]
        #[allow(unused)]
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
                "{0}/tiny/core/?.lua;{0}/tiny/core/?/init.lua;{0}/extensions/?.lua;{0}/extensions/?/init.lua",
                exec_folder.to_string_lossy()
            ),
        )?;

        register_all(&lua)?;

        let tiny_table = lua.create_table()?;

        // start context build
        // NOTE: must be construct before loading the extensions,
        let context_builder = ContextBuilder::new(&lua)?;

        // add to tiny table
        tiny_table.set("backends", context_builder)?;

        globals.set("tiny", &tiny_table)?;

        // load the core, require only
        let require_func: LuaFunction = globals.get("require")?;

        require_func.call::<()>("main")?;

        // load configurations (.tiny.json)
        let mut config_str = String::new();

        #[allow(unused)]
        {
            let mut json_path = Path::new(".tiny.json").to_path_buf();

            #[cfg(debug_assertions)]
            {
                json_path = exec_folder.join(".tiny.json");
            }

            let mut file =
                File::open(json_path).expect("Fail to load .tiny.json in current folder.");

            file.read_to_string(&mut config_str)?;
        }

        let configs: Configurations = serde_json::from_str(&config_str)?;

        // load extensions
        for ext_name in &configs.extensions {
            let ext_path = exec_folder.join("extensions").join(ext_name);

            debug!("loading extension from: {:?}", ext_path);

            if ext_path.exists() && ext_path.is_dir() {
                debug!("loaded extension: {}", ext_name);

                load_extension(&lua, ext_path)?;
            }
        }

        // load tools

        let context_builder = tiny_table.get::<LuaAnyUserData>("backends")?;
        let context =
            ContextBuilder::build(context_builder.call_method::<LuaTable>("collect", ())?)?;

        Ok(Self { lua, context })
    }

    pub fn run(&self) -> TinyResult<()> {
        Ok(())
    }
}
