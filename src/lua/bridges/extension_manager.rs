use std::{
    fs::File,
    io::Read,
    path::{self, Path},
};

use mlua::prelude::*;

/// We use this struct to hold all the extensions, instead expose them to globals or member of tiny
pub struct ExtensionManager {}

impl ExtensionManager {
    pub fn load(&self, lua: &Lua, path: String) -> LuaResult<()> {
        let extension_root_path = path::absolute(&path).unwrap();
        let extension_file_path = extension_root_path.join("extension.lua");
        let mut script = String::new();

        {
            let mut file = File::open(extension_file_path.to_str().unwrap())?;

            file.read_to_string(&mut script)?;
        }

        let globals = lua.globals();
        let pacakge_table = globals.get::<LuaTable>("package")?;
        let old_package_path = pacakge_table.get::<String>("path")?;

        let new_package_path = format!(
            "{};{}/?.lua;{}/?/init.lua",
            old_package_path,
            extension_root_path.to_str().unwrap(),
            extension_root_path.to_str().unwrap()
        );

        pacakge_table.set("path", new_package_path)?;

        let register_func: LuaFunction = lua.load(script).eval()?;

        pacakge_table.set("path", old_package_path)?;

        // TODO: pass configurations later
        // register_func.call::<()>(lua.create_table()?)?;
        println!("{:?}", register_func);

        Ok(())
    }
}

impl LuaUserData for ExtensionManager {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // create a method 'add_chat_client' to lua side, used to register a function to create a new instance of chat client
        // each constructor function can apply one lua table as configurations
        //
        // # Example
        //
        // ```lua
        //
        // function OpenAIChatClient(configs)
        //     return ChatClient(MyOpenAIProvider())
        // end
        //
        // extension_manager:add_chat_client("openai", OpenAIChatClient)
        //
        // ```
        // methods.add_meta_method_mut(
        //     "add_chat_client",
        //     |lua, manager, (name, constructor): (String, LuaFunction)| Ok(()),
        // );

        methods.add_method_mut("load", |lua, manager, path: String| {
            manager.load(lua, path)?;

            Ok(())
        });
    }
}

pub fn register(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();

    globals.set(
        "ExtensionManager",
        lua.create_function(|_, ()| Ok(ExtensionManager {}))?,
    )?;

    Ok(())
}
