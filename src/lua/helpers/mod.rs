use mlua::Lua;

use crate::core::TinyResult;

pub mod http;
pub mod json;
pub mod path;
pub mod string;

pub fn register_all(lua: &Lua) -> TinyResult<()> {
    http::register(lua)?;
    json::register(lua)?;
    path::register(lua)?;
    string::register(lua)?;

    Ok(())
}
