use crate::{core::tiny_loop, luaenv::lua::*};
use std::collections::HashMap;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{
    agent::types::{Tools, WChatOptions},
    core::{ChatOptions, Message, MessageChunk, TinyError, Tool, ToolCall, ToolCallResult},
    luaenv::env::LuaEnv,
    openai,
};

pub struct Session {
    pub messages: Vec<Message>,
}

pub struct Agent {
    lua_env: LuaEnv,
    chat_options: WChatOptions,
    tools: Vec<Tool>,                             // tools definition for llm
    tool_functions: HashMap<String, LuaFunction>, // function of each tool

    session: Session,

    chunk_tx: Sender<MessageChunk>,   // send streaming chunk
    chunk_rx: Receiver<MessageChunk>, // receive chunk

    tool_call_tx: Sender<ToolCall>,   // agent need to call a tool
    tool_call_rx: Receiver<ToolCall>, // tool executor receive

    tool_call_complete_tx: Sender<ToolCallResult>, // send result of tool call
    tool_call_complete_rx: Receiver<ToolCallResult>, // result from tool executor
}

impl Agent {
    pub fn new(name: &str, chat_options: WChatOptions, tools: Tools) -> Result<Self, TinyError> {
        let mut llm_tools: Vec<Tool> = Vec::new();
        let mut lua_tool_functions: HashMap<String, LuaFunction> = HashMap::new();

        for tool_pair in tools.0 {
            let tool_name = tool_pair.0.0.name.to_string();

            llm_tools.push(tool_pair.0.0);
            lua_tool_functions.insert(tool_name, tool_pair.0.1);
        }

        let (chunk_tx, chunk_rx) = mpsc::channel::<MessageChunk>(1024);

        // NOTE: now we only support calling 1 tool each time
        let (tool_call_tx, tool_call_rx) = mpsc::channel::<ToolCall>(1);
        let (tool_call_complete_tx, tool_call_complete_rx) = mpsc::channel::<ToolCallResult>(1);

        Ok(Agent {
            lua_env: LuaEnv::new(name).map_err(|e| TinyError::RuntimeError)?,
            session: Session { messages: vec![] },
            chat_options,
            tools: llm_tools,
            tool_functions: lua_tool_functions,
            chunk_tx,
            chunk_rx,
            tool_call_tx,
            tool_call_rx,
            tool_call_complete_tx,
            tool_call_complete_rx,
        })
    }
}

impl Agent {
    // pub async fn chat(&mut self, message: &str) -> Result<(), TinyError> {
    //     // start a task first
    //     self.session.messages = tiny_loop(
    //         &self.chat_options.0,
    //         self.session.messages,
    //         openai::chat, // TODO: use provider to switch later
    //         &self.tools,
    //         (self.tool_call_tx, self.tool_call_complete_rx.),
    //         self.chunk_tx,
    //     )
    //     .await
    //     .unwrap();

    //     Ok(())
    // }
}
