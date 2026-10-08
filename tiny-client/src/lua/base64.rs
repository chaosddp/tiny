use base64::Engine;
use mlua::prelude::*;

pub fn base64_encode(_: &Lua, s: String) -> LuaResult<String> {
    let encoded_str = base64::engine::general_purpose::STANDARD.encode(s.as_bytes());

    Ok(encoded_str)
}

pub fn base64_decode(_: &Lua, s: String) -> LuaResult<String> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(s).unwrap();

    Ok(String::from_utf8_lossy(&bytes).to_string())
}

pub fn register(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();

    let base64_table = lua.create_table()?;

    base64_table.set("encode", lua.create_function(base64_encode)?)?;
    base64_table.set("decode", lua.create_function(base64_decode)?)?;

    globals.set("base64", base64_table)?;

    Ok(())
}
