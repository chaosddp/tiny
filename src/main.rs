use log::error;

use crate::core::TinyResult;
use crate::core::error::Error as TinyError;
use crate::lua::agent::TinyAgent;
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

    let agent = TinyAgent::new();

    agent.init()?;

    agent.run()?;

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
