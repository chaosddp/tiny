#[cfg(all(feature = "async", feature = "sync"))]
compile_error!("feature \"async\" and feature \"sync\" cannot be enabled at the same time");

#[cfg(feature = "async")]
compile_error!("feature \"async\" is not completed.");

mod agent;
mod core;
mod luaenv;
mod openai;

use std::cell::Cell;
use std::io::Write;

use crate::agent::sync_agent::TinyAgent;
use crate::core::sync_impl::ChunkReceiver;
use crate::core::types::{ImageDetail, MessageChunk, TinyError};
use crate::openai::sync_impl::OpenaiClient;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ChunkState {
    NotStarted,
    Reasoning,
    Content,
    ToolCall,
}

struct ConsoleChunkReceiver {
    stdout: std::io::Stdout,
    state: Cell<ChunkState>,
}

impl ConsoleChunkReceiver {
    pub fn new() -> Self {
        ConsoleChunkReceiver {
            stdout: std::io::stdout(),
            state: Cell::new(ChunkState::NotStarted),
        }
    }
}

impl ChunkReceiver for ConsoleChunkReceiver {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    

    let mut agent = TinyAgent::new(
        "You are a helpful assistant",
        ".",
        Box::new(OpenaiClient::new()),
        Box::new(ConsoleChunkReceiver::new()),
    )?;

    agent.chat("what is the weather in Beijing.")?;

    Ok(())
}
