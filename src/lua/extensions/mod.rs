pub mod http;
pub mod json;
pub mod path;
pub mod string;

pub mod prelude {
    use mlua::{Lua, Result as LuaResult};

    use super::*;

    pub fn register_all(lua: &Lua) -> LuaResult<()> {
        json::register(lua)?;
        string::register(lua)?;
        http::register(lua)?;
        path::register(lua)?;

        Ok(())
    }
}
