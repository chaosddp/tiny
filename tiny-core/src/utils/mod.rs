pub mod base64;
pub mod http;
pub mod json;
pub mod path;
pub mod string;

use mlua::prelude::*;

pub fn register_all(lua: &Lua) -> LuaResult<()> {
    json::register(lua)?;
    string::register(lua)?;
    path::register(lua)?;
    base64::register(lua)?;
    http::register(lua)?;

    Ok(())
}
