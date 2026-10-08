use mlua::prelude::*;
use utf16string::{LittleEndian, WString};

pub fn to_utf8_bytes(lua: &Lua, s: String) -> LuaResult<LuaString> {
    lua.create_string(s.as_bytes())
}

pub fn to_utf16_bytes(lua: &Lua, s: String) -> LuaResult<LuaString> {
    let s0: WString<LittleEndian> = WString::from(&s);

    lua.create_string(s0.as_bytes())
}

pub fn register(lua: &Lua) -> LuaResult<()> {
    // we attach external functions to lua builtin string table
    let string_table = lua.globals().get::<LuaTable>("string")?;

    string_table.set(
        "to_bytes",
        lua.create_function(|lua, (s, encoding): (String, Option<String>)| {
            if let Some(encoding) = encoding {
                if encoding == "utf-16" {
                    to_utf16_bytes(lua, s)
                } else {
                    to_utf8_bytes(lua, s)
                }
            } else {
                to_utf8_bytes(lua, s)
            }
        })?,
    )?;

    Ok(())
}
