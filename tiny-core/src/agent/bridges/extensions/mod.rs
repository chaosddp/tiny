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

    // let globals = lua.globals();
    // let pacakge_table = globals.get::<LuaTable>("package")?;
    // let old_package_path = pacakge_table.get::<String>("path")?;

    // let new_package_path = format!(
    //     "{};{}/?.lua;{}/?/init.lua",
    //     old_package_path,
    //     extension_root_path.to_str().unwrap(),
    //     extension_root_path.to_str().unwrap()
    // );

    // pacakge_table.set("path", new_package_path)?;

    lua.load(script).exec()?;

    Ok(())
}
