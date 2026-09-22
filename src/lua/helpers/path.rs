use std::path::Path;

use mlua::prelude::*;

use crate::core::TinyResult;

pub fn is_path_exist(_: &Lua, p: String) -> LuaResult<bool> {
    Ok(Path::new(&p).exists())
}

pub fn is_file(_: &Lua, p: String) -> LuaResult<bool> {
    Ok(Path::new(&p).is_file())
}

pub fn is_dir(_: &Lua, p: String) -> LuaResult<bool> {
    Ok(Path::new(&p).is_dir())
}

pub fn get_path_info(lua: &Lua, p: String) -> LuaResult<LuaTable> {
    let path = Path::new(&p);
    let info_table = lua.create_table()?;

    info_table.set("is_dir", path.is_dir())?;
    info_table.set("is_file", path.is_file())?;
    info_table.set("exists", path.exists())?;
    info_table.set("extension", path.extension())?;
    info_table.set("file_name", path.file_name())?;
    info_table.set("base_name", path.file_stem())?;
    info_table.set("parent", path.parent())?;

    Ok(info_table)
}

pub fn register(lua: &Lua) -> TinyResult<()> {
    let path_table = lua.create_table()?;

    path_table.set("is_file", lua.create_function(is_file)?)?;
    path_table.set("is_dir", lua.create_function(is_dir)?)?;
    path_table.set("exists", lua.create_function(is_path_exist)?)?;
    path_table.set("get_path_info", lua.create_function(get_path_info)?)?;

    lua.globals().set("path", path_table)?;

    Ok(())
}
