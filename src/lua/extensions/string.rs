use base64::Engine;
use mlua::prelude::*;
use utf16string::{LittleEndian, WString};

use crate::core::TinyResult;

pub fn to_utf8_bytes(lua: &Lua, s: String) -> LuaResult<LuaString> {
    lua.create_string(s.as_bytes())
}

pub fn to_utf16_bytes(lua: &Lua, s: String) -> LuaResult<LuaString> {
    let s0: WString<LittleEndian> = WString::from(&s);

    lua.create_string(s0.as_bytes())
}

pub fn base64_encode(lua: &Lua, s: LuaString) -> LuaResult<LuaString> {
    let encoded_str = base64::engine::general_purpose::STANDARD.encode(s.as_bytes());

    lua.create_string(encoded_str.as_bytes())
}

// pub fn utf8_len(lua: &Lua, s: LuaString) -> LuaResult<usize> {
//     let s_bytes = s.as_bytes();
//     let s0 = String::from_utf8_lossy(&s_bytes);

//     Ok(s0.chars().)
// }

// pub fn utf8_substring(lua: &Lua, s: LuaString)->LuaResult<LuaString> {
//     let s_bytes = s.as_bytes();
//     let s0 = String::from_utf8_lossy(&s_bytes);
//     s[]
// }

pub fn register(lua: &Lua) -> TinyResult<()> {
    // we attach external functions to lua builtin string table
    let string_table = lua.globals().get::<LuaTable>("string")?;

    string_table.set("to_utf8_bytes", lua.create_function(to_utf8_bytes)?)?;
    string_table.set("to_utf16_bytes", lua.create_function(to_utf16_bytes)?)?;
    string_table.set("base64_encode", lua.create_function(base64_encode)?)?;
    // string_table.set("utf8_len", lua.create_function(utf8_len)?)?;

    Ok(())
}
