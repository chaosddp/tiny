use std::{fs::File, io::Read, path::Path};

use mlua::prelude::*;

use crate::lua::thread::value::LuaThreadValue;

/// create a new lua state, this state do not have thread support
pub struct LuaState {
    lua: Lua,
}

impl LuaState {
    pub fn new() -> LuaResult<Self> {
        let lua = Lua::new();

        super::register_all(&lua)?;

        Ok(Self { lua })
    }

    pub fn execute(&self, script: String) -> LuaResult<()> {
        self.lua.load(script).exec()?;

        Ok(())
    }

    pub fn eval(&self, script: String) -> LuaResult<LuaThreadValue> {
        let ret = self.lua.load(script).eval()?;

        Ok(ret)
    }
}

impl LuaUserData for LuaState {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("execute", |_, state, content_or_file: String| {
            let path = Path::new(&content_or_file);

            let mut script = String::new();

            if path.exists() {
                let mut file = File::open(path)?;
                file.read_to_string(&mut script)?;
            } else {
                script = content_or_file;
            }

            state.execute(script)?;

            Ok(())
        });

        methods.add_method("eval", |_, state, script: String| {
            let ret = state.eval(script)?;

            Ok(ret)
        });
    }
}

pub fn register(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();
    let state_table = lua.create_table()?;

    state_table.set(
        "newState",
        lua.create_function(|_, ()| {
            let state = LuaState::new();

            Ok(state)
        })?,
    )?;

    globals.set("lua", state_table)?;

    Ok(())
}
