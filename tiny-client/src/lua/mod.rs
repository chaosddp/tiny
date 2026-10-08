pub mod base64;
pub mod json;
pub mod path;
pub mod string;
pub mod http;
use mlua::prelude::*;

pub fn register_base(lua: &Lua) -> LuaResult<()> {
    json::register(lua)?;
    string::register(lua)?;
    path::register(lua)?;
    base64::register(lua)?;
    http::register(lua)?;

    Ok(())
}

pub fn register_all(lua: &Lua) -> LuaResult<()> {
    register_base(lua)?;

    Ok(())
}
