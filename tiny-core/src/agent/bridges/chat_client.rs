use std::io::{BufRead, BufReader};

use crate::core::{
    TinyResult,
    agent::{ChatClient, ChatOptions, ChunkReceiver},
    chat::{
        chunk::Chunk,
        messages::{AssistantMessage, ChatMessage},
        tool::Tool,
    },
    error::Error,
};
use log::debug;
use mlua::FromLua;
use mlua::prelude::*;
use reqwest::blocking::{Client, ClientBuilder};

pub struct LuaChatClient {
    lua: WeakLua,
    inner: LuaTable,
    client: Client,
}

impl LuaChatClient {
    /// Create a new LuaChatClient, and bind it to a lua table.
    ///
    /// It requires the table contains following methods
    /// 1. prepare_request(self, messages, options)->{url=..., body=..., headers=...} - construct http request options that contains body, headers
    /// 2. process_chunk(self, chunk_str)->{chunk_table} - parse the each SSE line into Chunk message, called if stream enabled
    /// 3. process_message(self, message_str)->{message_table} - parse the LLM response into assistant message, called if stream disabled
    /// 4. http_failed(self, status_code, response)->{strategy} - called on http request failed, and provide different strategy
    /// 5. parse_failed(self, response, error)->{strategy} -> called on chunk/message processing failed, and provide different strategy
    pub fn from_lua_table(lua: &Lua, table: LuaTable) -> LuaResult<Self> {
        for method_name in ["prepare_request", "process_chunk", "process_message"] {
            // check if the method exist
            let _method: LuaFunction = table.get(method_name)?;
        }

        let client = ClientBuilder::new().build().unwrap();

        Ok(Self {
            lua: lua.weak(),
            inner: table,
            client: client,
        })
    }
}

impl ChatClient for LuaChatClient {
    fn chat(
        &self,
        messages: &Vec<ChatMessage>,
        tools: Option<&Vec<Tool>>,
        options: &ChatOptions,
        chunk_receiver: Option<&Box<dyn ChunkReceiver>>,
    ) -> TinyResult<AssistantMessage> {
        let lua = self.lua.try_upgrade().ok_or(Error::InvalidLuaReference)?;

        let lua_messages = lua.create_table()?;

        for msg in messages {
            lua_messages.push(msg)?;
        }

        let lua_chat_options = options.clone().into_lua(&lua)?;
        let lua_tools = match tools {
            None => LuaValue::Nil,
            Some(ts) => {
                let t = lua.create_table()?;

                for d in ts {
                    t.push(d.clone())?;
                }

                LuaValue::Table(t)
            }
        };
        // ask provider for request things
        let (success, request_options) = self.inner.call_method::<(bool, LuaValue)>(
            "prepare_request",
            (lua_messages, lua_chat_options, lua_tools),
        )?;

        if !success {
            return Err(Error::RuntimeError(
                "Fail to create chat request options".into(),
            ));
        }

        let option_table = request_options.as_table().ok_or(Error::RuntimeError(
            "chat provider return invalid result, it should be a table with 'url', 'headers' and 'body' fields.".into(),
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
                        self.process_sse_chunks(chunk_receiver, resp)?
                    } else {
                        let resp_text = resp.text().unwrap();

                        // let assistant_message_table:LuaTable =  self.inner.call_method("process_message", resp_text)?;
                        let assistant_message =
                            self.inner.call_method("process_message", resp_text)?;

                        assistant_message
                    }
                }
                Err(e) => {
                    debug!("Lua client request error: {:?}", e);

                    return Err(Error::HttpError(
                        e.status().unwrap().as_u16(),
                        e.to_string(),
                    ));
                }
            },
            Err(e) => {
                // TODO: call failed function
                return Err(Error::RuntimeError(e.to_string()));
            }
        };

        Ok(message)
    }
}

impl LuaChatClient {
    fn process_sse_chunks(
        &self,
        chunk_receiver: Option<&Box<dyn ChunkReceiver>>,
        resp: reqwest::blocking::Response,
    ) -> TinyResult<AssistantMessage> {
        let mut content = String::new();
        let mut reasoning_content = String::new();
        let mut finish_reason = None;
        let mut tool_call = None;

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
                            .inner
                            .call_method::<LuaValue>("process_chunk", chunk_str)?;
                        let chunk: Chunk = Chunk::from_lua(chunk_table, &self.lua.upgrade())?;

                        // keep the content to construct AssistantMessage
                        if let Some(chunk_content) = &chunk.content {
                            content.push_str(&chunk_content);
                        }

                        if let Some(chunk_reasoning) = &chunk.reasoning {
                            reasoning_content.push_str(&chunk_reasoning);
                        }

                        finish_reason = chunk.finish_reason.clone();

                        if let Some(tc) = &chunk.tool_call {
                            tool_call = Some(vec![tc.clone()]);
                        }

                        if let Some(chunk_rc) = &chunk_receiver {
                            chunk_rc.recv(chunk)?;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(AssistantMessage {
            content: Some(content),
            reasoning: Some(reasoning_content),
            reasoning_details: None,
            tool_calls: tool_call,
            finish_reason: finish_reason.unwrap(),
            usage: None,
        })
    }
}
