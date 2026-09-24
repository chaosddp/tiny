use std::path;
use std::{fs::File, io::Read};

use log::error;
use mlua::prelude::*;

use crate::core::TinyResult;
use crate::core::error::Error as TinyError;
mod core;
mod lua;

fn pretty_lua_error(err: mlua::Error) {
    match err {
        mlua::Error::BadArgument {
            to,
            pos,
            name,
            cause,
        } => {
            error!(
                "Lua Error (BadArgument) - to: {to:?}, pos: {pos}, name: {name:?}, cause: {cause}"
            );
        }
        mlua::Error::RuntimeError(s) => {
            error!("Lua Error (RuntimeError) - {s:?}");
        }
        _ => error!("Lua Error - {}", err.to_string()),
    }
}

fn run() -> TinyResult<()> {
    env_logger::init();

    let lua = Lua::new();

    lua::extensions::prelude::register_all(&lua)?;
    lua::bridges::register_all(&lua)?;

    // preload all builtin modules
    let globals = lua.globals();
    let require_func = globals.get::<mlua::Function>("require")?;

    require_func.call::<()>("tiny")?;

    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    let extension_root_path = path::absolute(exe_dir)?.join("extensions");

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
    // let mut script = String::new();

    // {
    //     let mut fp = File::open("main.lua")?;
    //     fp.read_to_string(&mut script)?;
    // }

    // lua.load(script).exec()?;

    lua.load(
        r#"
        local run = require "tiny.main"

        print("run")

        run()
    "#,
    )
    .exec()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match run() {
        Err(e) => match e {
            TinyError::LuaError(e) => {
                pretty_lua_error(e);
            }
            _ => {
                print!("{:?}", e);
            }
        },
        Ok(_) => {}
    }

    Ok(())
}
