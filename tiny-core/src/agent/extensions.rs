use std::{
    fs::File, io::Read, path::{self, PathBuf},
};

use mlua::prelude::*;

pub fn load_extension(lua: &Lua, path: PathBuf) -> LuaResult<()> {
    let extension_root_path = path::absolute(&path).unwrap();
    let extension_file_path = extension_root_path.join("extension.lua");
    let mut script = String::new();

    {
        let mut file = File::open(extension_file_path.to_str().unwrap())?;

        file.read_to_string(&mut script)?;
    }

    lua.load(script).exec()?;

    Ok(())
}