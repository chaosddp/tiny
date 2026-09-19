use std::path::{Path, PathBuf};

use log::debug;

use crate::agent::sync_impl::{
    DefaultChunkReceiver, DefaultToolExecutor, LuaTraintObjectToolExecutor,
    LuaTraitObjectChunkReceiver,
};
use crate::{
    agent::types::{Tools, WChatOptions, WLuaTable},
    core::{
        ChatClient, ChatOptions, ChunkReceiver, Message, TinyError, Tool, ToolExecutor,
        UserMessage, tiny_loop,
    },
    luaenv::{env::LuaEnv, lua::*},
};

/// Session management
pub struct Session {
    pub messages: Vec<Message>,
}

impl Session {
    pub fn new(prompt: &str) -> Self {
        Session {
            messages: vec![Message::System(prompt.to_string())],
        }
    }
}

/// Agent that interactive with lua script.
/// It will try to load 'main.lua' file under folder '.tiny' for current working dir.
/// The 'main.lua' script will be used to initalize the agent, and provide tools and life-cycle management
///
/// # Example
///
/// ```lua
/// -- This agent will call this function automatically for configurations
/// function tiny.conf(t)
///     t.chat.provider = "openai" -- use openai compatible provider
///     t.chat.model = "qwen3.5"
///     t.chat.base_url = "http://localhost:11434/v1"
///     t.chat.api_key = "ollama"
///     t.chat.max_tokens = 10240000
///
///     t.chat.thinking.type = "enabled"
///     t.chat.thinking.budget_tokens = 8192
///     t.chat.reasoning_effort = "low" -- low, medium, hight, any other string
///
///     -- t.system_prompt = "myprompt"
///
///     t.tools = {
///         get_weather = {
///             desc = "get weather of specified city", -- descript of the tool
///             func = get_weather,                     -- real function to call
///             parameters = {
///                 city = {
///                     ["type"] = "string",
///                     desc = "city name",
///                     required = true
///                 }
///             }
///         }
///     }
///end
///
/// ```
#[allow(dead_code)]
pub struct TinyAgent {
    lua_env: LuaEnv,
    /// working directory
    work_dir: String,
    /// path to '.tiny' under working dir
    tiny_dir: PathBuf,
    /// chat options, initialized from lua script
    chat_options: WChatOptions,
    tools: Vec<Tool>, // tools definition for llm

    session: Session,

    chat_client: Box<dyn ChatClient>,
    tool_executor: Box<dyn ToolExecutor>,
    chunk_receiver: Box<dyn ChunkReceiver>,
}

impl TinyAgent {
    /// Create a new instance of [`TinyAgent`].
    ///
    /// If do not provide chunk_recever and tool_executor parameters, [`TinyAgent`] will use default implementations.
    ///
    /// The '.tiny' folder under 'work_dir' will be add the lua search path, so we can require customize lua modules.
    pub fn new(
        prompt: &str,
        work_dir: &str,
        chat_client: Box<dyn ChatClient>,
        chunk_receiver: Option<Box<dyn ChunkReceiver>>,
        tool_executor: Option<Box<dyn ToolExecutor>>,
    ) -> Result<Self, TinyError> {
        let env = LuaEnv::new("tiny").unwrap();

        let tiny_dir = Path::new(work_dir).join(".tiny");
        let entry_file = tiny_dir.join("main.lua");

        if !tiny_dir.exists() {
            return Err(TinyError::RuntimeError);
        }

        if !entry_file.exists() {
            return Err(TinyError::RuntimeError);
        }

        // update the package search path
        env.add_package_path(tiny_dir.join("?.lua").to_str().unwrap())?;

        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();

        // TODO: load builtin lua modules

        env.add_package_path(exe_dir.join("?.lua").to_str().unwrap())?;

        // add members
        env.add_member("tools", env.weak().upgrade().create_table()?)?;

        // load the entry script
        env.exec_script(entry_file.to_str().unwrap())?;

        let config_table = WLuaTable::from((&env.weak().upgrade(), ChatOptions::default())).0;

        // call the config function
        env.call::<()>("tiny.conf", &config_table)?;

        let tools: Vec<Tool> = Tools::from(
            &config_table
                .get::<LuaTable>("tools")
                .unwrap_or(env.weak().upgrade().create_table().unwrap()),
        )
        .0;

        // load trait object

        let chunk_receiver: Box<dyn ChunkReceiver> =
            match config_table.get::<LuaTable>("chunk_receiver") {
                Ok(chunk_receiver_lua_object) => Box::new(LuaTraitObjectChunkReceiver::new(
                    env.weak(),
                    chunk_receiver_lua_object,
                )?),
                _ => {
                    if let Some(p_receiver) = chunk_receiver {
                        p_receiver
                    } else {
                        Box::new(DefaultChunkReceiver::new())
                    }
                }
            };

        let tool_executor: Box<dyn ToolExecutor> =
            match config_table.get::<LuaTable>("tool_executor") {
                Ok(tool_execute_lua_object) => Box::new(LuaTraintObjectToolExecutor::new(
                    env.weak(),
                    tool_execute_lua_object,
                )?),
                _ => {
                    if let Some(p_executor) = tool_executor {
                        p_executor
                    } else {
                        Box::new(DefaultToolExecutor::new(env.weak()))
                    }
                }
            };

        Ok(TinyAgent {
            lua_env: env,
            session: Session::new(prompt),
            work_dir: work_dir.to_string(),
            tiny_dir: tiny_dir,
            chat_options: WChatOptions::from(&config_table),
            tool_executor,
            chunk_receiver,
            tools,
            chat_client,
        })
    }
}

/// Trait to convert value to related user message
pub trait ToUserMessage {
    fn to_message(self) -> Result<Message, TinyError>;
}

impl ToUserMessage for String {
    fn to_message(self) -> Result<Message, TinyError> {
        Ok(Message::User(UserMessage::Text(self)))
    }
}

impl ToUserMessage for &str {
    fn to_message(self) -> Result<Message, TinyError> {
        Ok(Message::user(self))
    }
}

impl ToUserMessage for Message {
    fn to_message(self) -> Result<Message, TinyError> {
        Ok(self)
    }
}

/// parameter for user message with image
#[allow(dead_code)]
pub struct Image<'a>(pub &'a str, pub &'a str);

impl<'a> ToUserMessage for Image<'a> {
    fn to_message(self) -> Result<Message, TinyError> {
        Message::image(self.0, self.1)
    }
}

impl TinyAgent {
    pub fn chat(&mut self, content: impl ToUserMessage) -> Result<(), TinyError> {
        let message = content.to_message()?;

        debug!("message to chat: {:?}", message);

        self.session.messages.push(message);

        tiny_loop(
            &self.chat_options.0,
            &mut self.session.messages,
            &self.chat_client, // TODO: use provider to switch later
            &self.tools,
            &self.tool_executor,
            &self.chunk_receiver,
        )?;

        Ok(())
    }
}
