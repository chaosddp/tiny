use std::{cell::Cell, io::Write};

use crate::luaenv::lua::*;
use log::debug;
use serde_json::Value as JsonValue;

use crate::core::{ChunkReceiver, MessageChunk, TinyError, ToolExecutor};

/// State of current chunk, we use this as breakpoint for different content
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ChunkState {
    NotStarted,
    Reasoning,
    Content,
    ToolCall,
}

/// Default chunk receiver that output content to console.
///
/// If use do not provide a chunk receiver from lua script, we will use this as default one.
pub(super) struct DefaultChunkReceiver {
    stdout: std::io::Stdout,
    state: Cell<ChunkState>,
}

/// Default tool executor, that will call tools in current lua environment in 'tiny.tools' object.
pub(super) struct DefaultToolExecutor {
    lua: WeakLua,
}

// TODO: can we make these a generate trait?

/// Wrap a lua object as a ChunkReceiver trait object
pub(super) struct LuaTraitObjectChunkReceiver {
    lua: WeakLua,
    lua_object: LuaTable,
}

/// Wrap a lua object as a ToolExecutor trait object
pub(super) struct LuaTraintObjectToolExecutor {
    lua: WeakLua,
    lua_object: LuaTable,
}

/// convert a Json value into Lua value recursively
fn json_to_lua(lua: &Lua, json_value: JsonValue) -> LuaValue {
    match json_value {
        JsonValue::Bool(v) => LuaValue::Boolean(v),
        JsonValue::Null => LuaValue::Nil,
        JsonValue::Number(n) => LuaValue::Number(n.as_f64().unwrap()),
        JsonValue::String(s) => s.into_lua(lua).unwrap(),
        JsonValue::Array(arr) => {
            // TODO: error handling?
            let lua_arr = lua.create_table().unwrap();

            for v in arr {
                lua_arr.push(json_to_lua(lua, v)).unwrap();
            }

            LuaValue::Table(lua_arr)
        }
        JsonValue::Object(o) => {
            let lua_obj = lua.create_table().unwrap();

            for (k, v) in o {
                lua_obj.set(k, json_to_lua(lua, v)).unwrap();
            }

            LuaValue::Table(lua_obj)
        }
    }
}

impl DefaultChunkReceiver {
    pub fn new() -> Self {
        DefaultChunkReceiver {
            stdout: std::io::stdout(),
            state: Cell::new(ChunkState::NotStarted),
        }
    }
}

impl ChunkReceiver for DefaultChunkReceiver {
    fn chunk(&self, chunk: MessageChunk) -> Result<(), TinyError> {
        let mut stdout = self.stdout.lock();

        match chunk {
            MessageChunk::Chunk {
                content,
                reasoning_content,
                tool_calls,
                tool_result,
            } => {
                if let Some(c) = content {
                    if self.state.get() != ChunkState::Content {
                        stdout.write(b"\n\n[Assistant]\n\n").unwrap();

                        self.state.set(ChunkState::Content);
                    }

                    stdout.write(c.as_bytes()).unwrap();
                }

                if let Some(rc) = reasoning_content {
                    if self.state.get() != ChunkState::Reasoning {
                        stdout.write(b"\n[Reasoning]\n\n").unwrap();

                        self.state.set(ChunkState::Reasoning);
                    }

                    stdout.write(rc.as_bytes()).unwrap();
                }

                if let Some(tool_calls) = tool_calls {
                    for tool_call in tool_calls {
                        stdout.write(b"\n\n[Tool call]\n\n").unwrap();

                        stdout.write(b"\nName: ").unwrap();
                        stdout.write(tool_call.name.as_bytes()).unwrap();
                        stdout.write(b"\n\n").unwrap();

                        if let Some(args_str) = tool_call.arguments {
                            stdout.write(b"\nArguments: ").unwrap();
                            stdout.write(args_str.as_bytes()).unwrap();
                            stdout.write(b"\n\n").unwrap();
                        }
                    }

                    self.state.set(ChunkState::ToolCall)
                }

                if let Some(tr) = tool_result {
                    stdout.write("\n\n[Tool result]\n\n".as_bytes()).unwrap();

                    stdout.write(tr.as_bytes()).unwrap();

                    self.state.set(ChunkState::ToolCall)
                }
            }
            MessageChunk::Error(e) => {
                println!("{}", e)
            }
        }

        stdout.flush().unwrap();

        Ok(())
    }
}

impl LuaTraitObjectChunkReceiver {
    pub fn new(lua: WeakLua, object: LuaTable) -> Result<LuaTraitObjectChunkReceiver, TinyError> {
        // validate if the object match the trait requirement
        let _ = object.get::<LuaFunction>("chunk").map_err(|_| {
            TinyError::InvalidLuaTraitObject(
                "Invalid ChunkReceiver trait object, it does not contain 'chunk' method."
                    .to_string(),
            )
        })?;

        Ok(LuaTraitObjectChunkReceiver {
            lua,
            lua_object: object,
        })
    }
}

impl ChunkReceiver for LuaTraitObjectChunkReceiver {
    fn chunk(&self, chunk: MessageChunk) -> Result<(), TinyError> {
        // transform the chunk into lua table
        // call specified function
        if let Some(lua) = self.lua.try_upgrade() {
            // TODO: shall we keep the table object, and reset the value each time to reduce object number?
            let chunk_table = lua.create_table_from([
                ("content", LuaValue::Nil),
                ("reasoning_content", LuaValue::Nil),
                ("tool_calls", LuaValue::Nil),
                ("tool_result", LuaValue::Nil),
            ])?;

            match chunk {
                MessageChunk::Chunk {
                    content,
                    reasoning_content,
                    tool_calls,
                    tool_result,
                } => {
                    if let Some(c) = content {
                        chunk_table.set("content", c)?;
                    }

                    if let Some(rc) = reasoning_content {
                        chunk_table.set("reasoning_content", rc)?;
                    }

                    if let Some(tool_call_list) = tool_calls {
                        let tool_call_list_table = lua.create_table()?;

                        for tool_call in tool_call_list {
                            let tool_call_table = lua.create_table()?;

                            tool_call_table.set("name", tool_call.name)?;
                            tool_call_table.set("id", tool_call.id)?;
                            tool_call_table.set("index", tool_call.index)?;
                            tool_call_table.set("arguments", tool_call.arguments)?;

                            tool_call_list_table.push(tool_call_table)?;
                        }

                        chunk_table.set("tool_calls", tool_call_list_table)?;
                    }

                    if let Some(tool_result_str) = tool_result {
                        chunk_table.set("tool_result", tool_result_str)?;
                    }
                }
                _ => {}
            }

            self.lua_object.call_method::<()>("chunk", chunk_table)?;

            Ok(())
        } else {
            Err(TinyError::InvalidLuaReference)
        }
    }
}

impl DefaultToolExecutor {
    pub fn new(lua: WeakLua) -> Self {
        DefaultToolExecutor { lua }
    }
}

impl ToolExecutor for DefaultToolExecutor {
    fn exec(&self, name: &str, id: &str, tool_args: Option<&str>) -> Result<String, TinyError> {
        debug!("recieve tool call ({}): {}({:?})", name, id, tool_args);

        let tool_func_full_name = format!("tiny.tools.{}", name);
        let tool_func = self
            .lua
            .upgrade()
            .globals()
            .get_path::<LuaFunction>(&tool_func_full_name)?;

        match tool_args {
            Some(args_str) => match serde_json::from_str(args_str) {
                Ok(args_json_value) => {
                    let args = json_to_lua(&self.lua.upgrade(), args_json_value);

                    let ret = tool_func.call::<String>(args)?;

                    return Ok(ret);
                }
                _ => {
                    return Err(TinyError::InvalidJsonObject(args_str.to_string()));
                }
            },
            _ => {
                return Ok(tool_func.call::<String>(())?);
            }
        }
    }
}

impl LuaTraintObjectToolExecutor {
    pub fn new(lua: WeakLua, object: LuaTable) -> Result<Self, TinyError> {
        // validate if the object match the trait requirement
        let _ = object.get::<LuaFunction>("exec").map_err(|_| {
            TinyError::InvalidLuaTraitObject(
                "Invalid ToolExecutor trait object, it does not contain 'exec' method.".to_string(),
            )
        })?;

        Ok(LuaTraintObjectToolExecutor {
            lua,
            lua_object: object,
        })
    }
}

impl ToolExecutor for LuaTraintObjectToolExecutor {
    fn exec(&self, name: &str, id: &str, tool_args: Option<&str>) -> Result<String, TinyError> {
        debug!("recieve tool call ({}): {}({:?})", name, id, tool_args);

        // we just convert the arguments into a nested table, then call exec function of the lua object
        // it knows where to find the tool functions
        match tool_args {
            Some(args_str) => {
                if let Some(lua) = self.lua.try_upgrade() {
                    match serde_json::from_str::<JsonValue>(args_str) {
                        Ok(args_json_value) => {
                            let args = json_to_lua(&lua, args_json_value);

                            let ret = self
                                .lua_object
                                .call_method::<String>("exec", (name, args))?;

                            return Ok(ret);
                        }
                        Err(_) => {
                            return Err(TinyError::InvalidJsonObject(args_str.to_string()));
                        }
                    }
                } else {
                    return Err(TinyError::InvalidLuaReference);
                }
            }
            _ => {
                return Ok(self
                    .lua_object
                    .call_method::<String>("exec", (name, LuaValue::Nil))?);
            }
        };
    }
}
