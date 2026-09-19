use std::{cell::Cell, io::Write};

use crate::luaenv::lua::*;

use crate::core::{ChunkReceiver, MessageChunk, TinyError};

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
pub(super) struct DefaultConsoleChunkReceiver {
    stdout: std::io::Stdout,
    state: Cell<ChunkState>,
}

impl DefaultConsoleChunkReceiver {
    pub fn new() -> Self {
        DefaultConsoleChunkReceiver {
            stdout: std::io::stdout(),
            state: Cell::new(ChunkState::NotStarted),
        }
    }
}

impl ChunkReceiver for DefaultConsoleChunkReceiver {
    fn chunk(&self, chunk: MessageChunk) -> Result<(), TinyError> {
        let mut stdout = self.stdout.lock();

        match chunk {
            MessageChunk::Chunk {
                content,
                reasoning_content,
                tool_calls: _,
            } => {
                if let Some(c) = content {
                    if self.state.get() != ChunkState::Content {
                        stdout.write("\n\n[Assistant]\n\n".as_bytes()).unwrap();

                        self.state.set(ChunkState::Content);
                    }

                    stdout.write(c.as_bytes()).unwrap();
                }

                if let Some(rc) = reasoning_content {
                    if self.state.get() != ChunkState::Reasoning {
                        stdout.write("\n[🤔Reasoning]\n\n".as_bytes()).unwrap();

                        self.state.set(ChunkState::Reasoning);
                    }

                    stdout.write(rc.as_bytes()).unwrap();
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

/// Wrap a lua object as a ChunkReceiver object
pub(super) struct LuaChunkReceiverWrapper {
    lua: WeakLua,
    lua_object: LuaTable,
}

impl LuaChunkReceiverWrapper {
    pub fn new(lua: WeakLua, object: LuaTable) -> Result<LuaChunkReceiverWrapper, TinyError> {
        // validate if the object match the trait requirement
        let _ = object.get::<LuaFunction>("chunk").map_err(|_| {
            TinyError::InvalidLuaTraitObject(
                "Invalid ChunkReceiver trait object, it does not contain 'chunk' method."
                    .to_string(),
            )
        })?;

        Ok(LuaChunkReceiverWrapper {
            lua,
            lua_object: object,
        })
    }
}

impl ChunkReceiver for LuaChunkReceiverWrapper {
    fn chunk(&self, chunk: MessageChunk) -> Result<(), TinyError> {
        // transform the chunk into lua table
        // call specified function
        if let Some(lua) = self.lua.try_upgrade() {
            let chunk_table = lua.create_table_from([
                ("content", LuaValue::Nil),
                ("reasoning_content", LuaValue::Nil),
                ("tool_call", LuaValue::Nil),
            ])?;

            match chunk {
                MessageChunk::Chunk {
                    content,
                    reasoning_content,
                    tool_calls: _,
                } => {
                    if let Some(c) = content {
                        chunk_table.set("content", c)?;
                    }

                    if let Some(rc) = reasoning_content {
                        chunk_table.set("reasoning_content", rc)?;
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
