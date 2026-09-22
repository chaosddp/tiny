use mlua::Lua;

use crate::core::TinyResult;

pub mod chat_client;
pub mod tool_executor;

pub fn register_all(lua: &Lua) -> TinyResult<()> {
    chat_client::register(lua)?;
    tool_executor::register(lua)?;

    Ok(())
}
