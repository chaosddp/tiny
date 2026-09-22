use mlua::Lua;

use crate::core::TinyResult;

pub mod chat_client;

pub fn register_all(lua: &Lua) -> TinyResult<()> {
    chat_client::register(lua)?;

    Ok(())
}