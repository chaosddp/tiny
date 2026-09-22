use std::{fs::File, io::Read};

use log::error;
use mlua::Lua;

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
        mlua::Error::RuntimeError(s)=> {
            error!(
                "Lua Error (RuntimeError) - {s:?}"
            );
        }
        _ => error!("Lua Error - {}", err.to_string()),
    }
}

fn run() -> TinyResult<()> {
    env_logger::init();

    let lua = Lua::new();

    lua::extensions::prelude::register_all(&lua)?;
    lua::bridges::register_all(&lua)?;

    let mut script = String::new();

    {
        let mut fp = File::open("main.lua")?;
        fp.read_to_string(&mut script)?;
    }

    lua.load(script).exec()?;

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
