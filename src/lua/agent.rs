use std::{fs::File, io::Read};

use crate::{core::TinyResult, lua::utils::json::json_to_lua};

use mlua::prelude::*;

pub struct TinyAgent {
    lua: Lua,
}

impl TinyAgent {
    pub fn new() -> Self {
        let lua = unsafe { Lua::unsafe_new() };

        Self { lua }
    }

    pub fn init(&self) -> TinyResult<()> {
        // register our lua utilities and bridge wrappers
        crate::lua::utils::prelude::register_all(&self.lua)?;
        crate::lua::bridges::register_all(&self.lua)?;

        // preload all builtin modules
        let globals = self.lua.globals();
        let require_func = globals.get::<mlua::Function>("require")?;

        require_func.call::<()>("tiny")?;

        Ok(())
    }

    pub fn run(&self) -> TinyResult<()> {
        let globals = self.lua.globals();
        let require_func = globals.get::<mlua::Function>("require")?;

        let run_func = require_func.call::<LuaFunction>("tiny.main")?;

        // add tiny table
        let tiny_table = self.lua.create_table()?;
        let config_table = self.lua.create_table()?;

        config_table.set("models", self.lua.create_table()?)?;
        config_table.set(
            "extensions",
            self.lua.create_sequence_from(["default", "openai"])?,
        )?;

        tiny_table.set("configs", config_table)?;

        globals.set("tiny", tiny_table)?;

        // open .tiny.json for configurations
        let mut tiny_config_str = String::new();

        {
            let mut file = File::open(".tiny.json")?;

            file.read_to_string(&mut tiny_config_str)?;
        }

        let config_json_value: serde_json::Value = serde_json::from_str(&tiny_config_str).unwrap();

        let config_lua_table = json_to_lua(&self.lua, config_json_value);

        run_func.call::<()>(config_lua_table)?;
        Ok(())
    }
}
