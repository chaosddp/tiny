#[cfg(all(feature = "async", feature = "sync"))]
compile_error!("feature \"async\" and feature \"sync\" cannot be enabled at the same time");

mod agent;
mod core;
mod luaenv;
mod openai;

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::io::Write;

use log::debug;

use crate::agent::types::{Tools, WChatOptions, WLuaTable};
use crate::core::sync_impl::tiny_loop;
use crate::core::{ChatOptions, Message, MessageChunk, TinyError, Tool, UserMessage};
use crate::luaenv::{env::LuaEnv, lua::*};

fn on_chunk(msg: &str) {
    print!("{}", msg);

    std::io::stdout().flush().unwrap();
}

struct A {}

impl A {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let env = LuaEnv::new("tiny").unwrap();

    // load the entry script
    env.exec_script(".tiny/main.lua")?;

    let config_table = WLuaTable::from((&env.weak().upgrade(), ChatOptions::default())).0;

    // call the config function
    env.call::<()>("tiny.conf", &config_table)?;

    let options = WChatOptions::from(&config_table).0;

    let messages = vec![
        Message::SystemMessage("You are a helpful assistant.".to_string()),
        Message::UserMessage(UserMessage::Text("what is the weather in Beijing?".into())),
    ];

    let lua_tools: Vec<agent::types::LuaFuncTool> = Tools::from(
        &config_table
            .get::<LuaTable>("tools")
            .unwrap_or(env.weak().upgrade().create_table().unwrap()),
    )
    .0;

    let mut tools: Vec<Tool> = Vec::new();
    let mut lua_tool_functions: HashMap<String, LuaFunction> = HashMap::new();

    for tool_pair in lua_tools {
        let tool_name = tool_pair.0.0.name.to_string();

        debug!("lua tool function: {} -> {:?}", tool_name, tool_pair.0.1);

        tools.push(tool_pair.0.0);
        lua_tool_functions.insert(tool_name, tool_pair.0.1);
    }

    tiny_loop(
        &options,
        messages,
        openai::sync_impl::chat,
        &tools,
        |name, id, tool_args| {
            debug!("recieve tool call ({}): {}({:?})", name, id, tool_args);

            if name.len() > 0 {
                if let Some(func) = lua_tool_functions.get(name) {
                    debug!("found function: {:?}", func.info());

                    if let Some(args) = tool_args {
                        // parse it to json object, then to lua table
                        match serde_json::from_str::<JsonValue>(&args) {
                            Ok(args_json_value) if args_json_value.is_object() => {
                                // we use the first level as parameters
                                // let mut args_map: HashMap<String, LuaValue> = HashMap::new();
                                let args_table = env.weak().upgrade().create_table().unwrap();

                                for (k, v) in args_json_value.as_object().unwrap() {
                                    // TODO: support nested parameter type
                                    match v {
                                        JsonValue::Bool(b) => {
                                            args_table.set(k.clone(), *b).unwrap()
                                        }
                                        JsonValue::Null => {
                                            args_table.set(k.clone(), LuaValue::Nil).unwrap()
                                        }
                                        JsonValue::Number(n) => {
                                            args_table.set(k.clone(), n.as_f64()).unwrap()
                                        }
                                        JsonValue::String(s) => {
                                            args_table.set(k.clone(), s.clone()).unwrap()
                                        }
                                        _ => {}
                                    }
                                }

                                match func.call::<String>(args_table) {
                                    Ok(ret) => return Ok(ret),
                                    Err(e) => {
                                        return Ok(format!(
                                            "fail to call the tool: {}",
                                            e.to_string()
                                        ));
                                    }
                                }
                            }
                            _ => {
                                return Ok(
                                    "Invalid tool call parameters, need a valid json object."
                                        .to_string(),
                                );
                            }
                        }
                    } else {
                        let ret = func
                            .call::<String>(())
                            .map_err(|e| TinyError::RuntimeError)?;

                        return Ok(ret);
                    }
                }
            } else {
                debug!(
                    "avaiable tools: {:?}",
                    lua_tool_functions
                        .keys()
                        .map(|k| k.clone())
                        .collect::<String>()
                );
            }

            Err(TinyError::RuntimeError)
        },
        |chunk| {
            match chunk {
                MessageChunk::Chunk {
                    content,
                    reasoning_content,
                    tool_calls,
                } => {
                    if let Some(c) = content {
                        print!("{}", c);
                        std::io::stdout().flush().unwrap();
                    }

                    if let Some(rc) = reasoning_content {
                        print!("{}", rc);
                        std::io::stdout().flush().unwrap();
                    }

                    if let Some(tc_list) = tool_calls {
                        println!("\n{:?}", tc_list);
                    }
                }
                MessageChunk::Error(e) => {
                    println!("{}", e)
                }
            }

            Ok(())
        },
    )?;

    Ok(())
}
