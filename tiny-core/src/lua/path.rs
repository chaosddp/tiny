use std::path::{self, Path};

use mlua::prelude::*;

pub fn is_path_exist(_: &Lua, p: String) -> LuaResult<bool> {
    Ok(Path::new(&p).exists())
}

pub fn is_file(_: &Lua, p: String) -> LuaResult<bool> {
    Ok(Path::new(&p).is_file())
}

pub fn is_dir(_: &Lua, p: String) -> LuaResult<bool> {
    Ok(Path::new(&p).is_dir())
}

pub fn absolute(_: &Lua, p: String) -> LuaResult<String> {
    let abp =
        path::absolute(p).map_err(|_| mlua::Error::RuntimeError("invalid path: {}".to_string()))?;

    Ok(abp.to_str().unwrap().to_string())
}

pub fn source_base_dir(_: &Lua, _: ()) -> LuaResult<String> {
    let exe_path = std::env::current_exe()?;
    let base_path = exe_path.parent().unwrap();

    Ok(base_path.to_string_lossy().to_string())
}

pub fn working_dir(_: &Lua, _: ()) -> LuaResult<String> {
    let cur_dir = std::env::current_dir()?;

    Ok(cur_dir.to_string_lossy().to_string())
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

pub fn glob(lua: &Lua, pattern: String) -> LuaResult<LuaTable> {
    let result = lua.create_table()?;

    for entry in glob::glob(&pattern).map_err(|e| mlua::Error::RuntimeError(e.to_string()))? {
        if let Ok(paths) = entry {
            result.push(paths.to_string_lossy().to_string())?;
        }
    }

    Ok(result)
}

pub fn register(lua: &Lua) -> LuaResult<()> {
    let path_table = lua.create_table()?;

    path_table.set("is_file", lua.create_function(is_file)?)?;
    path_table.set("is_dir", lua.create_function(is_dir)?)?;
    path_table.set("exists", lua.create_function(is_path_exist)?)?;
    path_table.set("get_path_info", lua.create_function(get_path_info)?)?;
    path_table.set("absolute", lua.create_function(absolute)?)?;
    path_table.set("source_base_dir", lua.create_function(source_base_dir)?)?;
    path_table.set("working_dir", lua.create_function(working_dir)?)?;
    path_table.set("glob", lua.create_function(glob)?)?;

    lua.globals().set("path", path_table)?;

    Ok(())
}
