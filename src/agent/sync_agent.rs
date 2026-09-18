use log::debug;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{
    agent::types::{LuaFuncTool, Tools, WChatOptions, WLuaTable},
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

/// Lua tool executor
struct LuaToolExecutor {
    lua_functions: HashMap<String, LuaFunction>,
    lua: WeakLua,
}

impl LuaToolExecutor {
    pub fn new(lua: WeakLua, functions: HashMap<String, LuaFunction>) -> Self {
        LuaToolExecutor {
            lua_functions: functions,
            lua,
        }
    }
}

impl ToolExecutor for LuaToolExecutor {
    fn exec(&self, name: &str, id: &str, tool_args: Option<&str>) -> Result<String, TinyError> {
        debug!("recieve tool call ({}): {}({:?})", name, id, tool_args);

        if name.len() > 0 {
            if let Some(func) = self.lua_functions.get(name) {
                debug!("found function: {:?}", func.info());

                if let Some(args) = tool_args {
                    // parse it to json object, then to lua table
                    match serde_json::from_str::<JsonValue>(&args) {
                        Ok(args_json_value) if args_json_value.is_object() => {
                            // we use the first level as parameters
                            if let Some(lua) = self.lua.try_upgrade() {
                                let args_table = lua.create_table().unwrap();

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
                        }
                        _ => {
                            return Ok("Invalid tool call parameters, need a valid json object."
                                .to_string());
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
                self.lua_functions
                    .keys()
                    .map(|k| k.clone())
                    .collect::<String>()
            );
        }

        Err(TinyError::RuntimeError)
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
    pub fn new(
        prompt: &str,
        work_dir: &str,
        chat_client: Box<dyn ChatClient>,
        chunk_receiver: Box<dyn ChunkReceiver>,
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

        // load the entry script
        env.exec_script(entry_file.to_str().unwrap())
            .map_err(|e| TinyError::RuntimeError)?;

        let config_table = WLuaTable::from((&env.weak().upgrade(), ChatOptions::default())).0;

        // call the config function
        env.call::<()>("tiny.conf", &config_table)
            .map_err(|e| TinyError::RuntimeError)?;

        let lua_tools: Vec<LuaFuncTool> = Tools::from(
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

        let lua_ref = env.weak();

        Ok(TinyAgent {
            lua_env: env,
            session: Session::new(prompt),
            work_dir: work_dir.to_string(),
            tiny_dir: tiny_dir,
            chat_options: WChatOptions::from(&config_table),
            tool_executor: Box::new(LuaToolExecutor::new(lua_ref, lua_tool_functions)),
            tools,
            chat_client,
            chunk_receiver,
        })
    }
}

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

pub struct Image<'a>(pub &'a str, pub &'a str);

impl<'a> ToUserMessage for Image<'a> {
    fn to_message(self) -> Result<Message, TinyError> {
        Message::image(self.0, self.1)
    }
}

impl TinyAgent {
    pub fn chat(&mut self, content: impl ToUserMessage) -> Result<(), TinyError> {
        let message = content.to_message()?;

        // debug!("message to chat: {:?}", message);

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
