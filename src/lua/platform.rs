use mlua::prelude::*;
use sysinfo::System;

pub fn get_platform_info(lua: &Lua, _: ()) -> LuaResult<LuaTable> {
    lua.create_table_from([
        ("system_name", System::name()),
        ("kernel_version", System::kernel_version()),
        ("system_version", System::os_version()),
    ])
}

pub fn register(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();
    let platform_table = lua.create_table()?;

    platform_table.set("info", lua.create_function(get_platform_info)?)?;

    globals.set("platform", platform_table)?;

    Ok(())
}