use std::io::{BufRead, BufReader};

use log::debug;
use mlua::prelude::*;
use reqwest::blocking::{Client, ClientBuilder};

use crate::core::{TinyResult, error::Error as TinyError};

/// General wrapper to access llm chat interface, it need a provider object to prepare request things and process SSE chunk
///
/// After register, it will create a global lua function named 'ChatClient', which require a lua object to process differences.
///
/// # Example in lua side
///
/// ```lua
///
/// --at lua side it have following definitions
///
/// ---process differences for proiders
/// ---@class ChatProvider
/// local ChatProvider = {}
///
/// ---Create a new ChatClient for a provider
/// ---@param provider ChatProvider
/// ---@return IChatClient
/// function ChatClient(provider) end
///
///
/// -- we have implement a customize provider table to process request option and SSE chunk, following is a simple openai compatible object
///
/// ---@type ChatProvider
/// local OpenAIChatProvider = {}
///
/// function OpenAIChatProvider:request(messages, tools, options)
///     local url = options.base_url .. "/chat/completions"
///
///     local body = {
///         model = options.model,
///         stream = options.stream,
///         max_tokens = options.max_tokens,
///         messages = messages
///     }
///
///     ---@type RequestOptions
///     local request_options = {
///         url = url,
///         body = json.dump(body),
///         headers = {
///             Authorization = "Bearer " .. options.api_key,
///             ["Content-Type"] = "application/json"
///         }
///     }
///
///     return true, request_options
/// end
///
/// function OpenAIChatProvider:chunk(chunk_str)
///     local chunk_message = json.load(chunk_str)
///
///     local chunk = {}
///
///     if chunk_message.choices and #chunk_message.choices > 0 then
///         local first_choice = chunk_message.choices[1]
///         chunk.content = first_choice.delta.content
///         chunk.reasoning = first_choice.delta.reasoning
///     end
///
///     return chunk
/// end
///
/// --- then this is the chat client that can chat with openai compatible interface
/// local chat_client = ChatClient(OpenAIChatProvider)
///
/// local message = chat_client:chat(
/// {
///     {
///         role = "system",
///         content = "You are a helpful assistant"
///     },
///     {
///         role = "user",
///         content = "why is the sky blue?"
///     }
/// },
/// {}, -- tools
/// {
///     model = "qwen3.5",
///     base_url = "http://localhost:11434/v1",
///     api_key = "Ollama",
///     stream = true,
///     max_tokens = 1024000
/// }, -- options
/// nil -- chunk receiver can be nil
/// )
/// ```
pub struct LuaChatClient {
    lua: WeakLua,
    chat_provider: LuaTable,
    client: Client,
}

impl LuaChatClient {
    /// Create a new LuaChatClient with Lua state reference, and chat provider lua object
    pub fn new(lua: WeakLua, chat_provider: LuaTable) -> Self {
        let client = ClientBuilder::new().build().unwrap();

        Self {
            lua,
            chat_provider,
            client,
        }
    }

    pub fn chat(
        &self,
        messages: LuaTable,
        options: LuaTable,
        tools: Option<LuaTable>,
        chunk_receiver: Option<LuaTable>,
    ) -> TinyResult<LuaTable> {
        let lua = self
            .lua
            .try_upgrade()
            .ok_or(TinyError::InvalidLuaReference)?;

        // ask provider for request things
        let (success, request_options) = self
            .chat_provider
            .call_method::<(bool, LuaValue)>("request", (messages, options, tools))?;

        if !success {
            return Err(TinyError::RuntimeError(
                "Fail to create chat request options".to_string(),
            ));
        }

        let option_table = request_options.as_table().ok_or(TinyError::RuntimeError(
            "chat provider return invalid result, it should be a table with 'url', 'headers' and 'body' fields.".to_string(),
        ))?;

        let url = option_table.get::<String>("url")?;
        let headers = option_table.get::<LuaTable>("headers")?;
        let body = option_table.get::<String>("body")?;

        let mut request_builder = self.client.post(url);

        for pair in headers.pairs::<String, String>() {
            if let Ok((k, v)) = pair {
                request_builder = request_builder.header(k, v);
            }
        }

        request_builder = request_builder.body(body);

        let message = match request_builder.send() {
            Ok(reaponse) => match reaponse.error_for_status() {
                Ok(resp) => {
                    let content_type = resp.headers().get("Content-Type").unwrap();
                    debug!("Current response content type: {:?}", content_type);

                    if content_type == "text/event-stream" {
                        self.process_sse_chunks(&lua, chunk_receiver, resp)?
                    } else {
                        let resp_text = resp.text().unwrap();

                        debug!("response text: \n{0}", &resp_text);

                        self.chat_provider
                            .call_method::<LuaTable>("message", resp_text)?
                    }
                }
                Err(e) => {
                    debug!("Lua client request error: {:?}", e);

                    return Err(TinyError::HttpError(e.status().unwrap(), e.to_string()));
                }
            },
            Err(e) => {
                // TODO: call failed function
                return Err(TinyError::RuntimeError(e.to_string()));
            }
        };

        Ok(message)
    }

    fn process_sse_chunks(
        &self,
        lua: &Lua,
        chunk_receiver: Option<LuaTable>,
        resp: reqwest::blocking::Response,
    ) -> Result<LuaTable, TinyError> {
        let mut content = String::new();
        let mut reasoning_content = String::new();
        let mut finish_reason = None;

        let reader = BufReader::new(resp);
        for line_ret in reader.lines() {
            match line_ret {
                Ok(line) => {
                    if line.starts_with("data: ") {
                        let chunk_str = line.strip_prefix("data:").unwrap().trim();

                        if chunk_str == "[DONE]" {
                            break;
                        }

                        let chunk_table = self
                            .chat_provider
                            .call_method::<LuaTable>("chunk", chunk_str)?;

                        // keep the content to construct AssistantMessage
                        if let Ok(chunk_content) = chunk_table.get::<String>("content") {
                            content.push_str(&chunk_content);
                        }

                        if let Ok(chunk_reasoning) = chunk_table.get::<String>("reasoning") {
                            reasoning_content.push_str(&chunk_reasoning);
                        }

                        if let Ok(chunk_finish_reason) = chunk_table.get::<String>("finish_reason")
                        {
                            finish_reason = Some(chunk_finish_reason);
                        }

                        if let Some(chunk_rc) = &chunk_receiver {
                            chunk_rc.call_method::<()>("receive", chunk_table)?;
                        }
                    }
                }
                _ => {}
            }
        }

        // our assistant message from chunks
        let message = lua.create_table()?;

        message.set("role", "assistant")?;
        message.set("content", content)?;
        message.set("reasoning_content", reasoning_content)?;
        message.set("finish_reason", finish_reason)?;

        Ok(message)
    }
}

// add methods to lua side
impl LuaUserData for LuaChatClient {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "chat",
            |_, client, (messages, options, tools, chunk_receiver)| {
                let message = client
                    .chat(messages, options, tools, chunk_receiver)
                    .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;

                Ok(message)
            },
        );
    }
}

pub fn register(lua: &Lua) -> TinyResult<()> {
    let globals = lua.globals();

    globals.set(
        "ChatClient",
        lua.create_function(|lua, provider_obj: LuaTable| {
            Ok(LuaChatClient::new(lua.weak(), provider_obj))
        })?,
    )?;

    Ok(())
}
