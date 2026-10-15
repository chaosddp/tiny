use std::io::{BufRead, BufReader};

use crate::core::*;
use log::debug;
use mlua::FromLua;
use mlua::prelude::*;
use reqwest::blocking::{Client, ClientBuilder};

/// used to implement basic chatting interface, it will fetch a chat provider by specified client name, then use it to process http request and response.
///
/// It requires the providers contains following methods
/// 1. prepare_request(self, messages, options)->{url=..., body=..., headers=...} - construct http request options that contains body, headers
/// 2. process_chunk(self, chunk_str)->{chunk_table} - parse the each SSE line into Chunk message, called if stream enabled
/// 3. process_message(self, message_str)->{message_table} - parse the LLM response into assistant message, called if stream disabled
/// 4. http_failed(self, status_code, response)->{strategy} - called on http request failed, and provide different strategy
/// 5. parse_failed(self, response, error)->{strategy} -> called on chunk/message processing failed, and provide different strategy
pub struct LuaChatClient {
    lua: WeakLua,
    inner: LuaTable, // key if the client name, value if the chat provider
    client: Client,
}

impl LuaChatClient {
    pub fn new(lua: &Lua) -> LuaResult<Self> {
        let client = ClientBuilder::new().build().unwrap();
        let inner: LuaTable = lua.globals().get_path("tiny.chat_clients")?;

        Ok(Self {
            lua: lua.weak(),
            client,
            inner,
        })
    }
}

impl LuaChatClient {
    pub fn chat<F>(
        &self,
        client: &str,
        messages: &Vec<ChatMessage>,
        tools: Option<&Vec<Tool>>,
        options: &ChatOptions,
        chunk_receiver: Option<F>,
    ) -> TinyResult<AssistantMessage>
    where
        F: Fn(Chunk) -> TinyResult<()>,
    {
        let lua = self
            .lua
            .try_upgrade()
            .ok_or(TinyError::InvalidLuaReference)?;
        let inner = self.inner.get::<LuaTable>(client)?;

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
        let (success, request_options) = inner.call_method::<(bool, LuaValue)>(
            "prepare_request",
            (lua_messages, lua_chat_options, lua_tools),
        )?;

        if !success {
            return Err(TinyError::RuntimeError(
                "Fail to create chat request options".into(),
            ));
        }

        let option_table = request_options.as_table().ok_or(TinyError::RuntimeError(
            "chat provider return invalid result, it should be a table with 'url', 'headers' and 'body' fields.".into(),
        ))?;

        let url = option_table.get::<String>("url")?;
        let headers = option_table.get::<LuaTable>("headers")?;
        let body = option_table.get::<String>("body")?;

        debug!("request body: {}", body);

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
                        self.process_sse_chunks(inner, chunk_receiver, resp)?
                    } else {
                        let resp_text = resp.text().unwrap();

                        inner.call_method("process_message", resp_text)?
                    }
                }
                Err(e) => {
                    debug!("Lua client request error: {:?}", e);

                    return Err(TinyError::HttpError(
                        e.status().unwrap().as_u16(),
                        e.to_string(),
                    ));
                }
            },
            Err(e) => {
                // TODO: call failed function
                return Err(TinyError::RuntimeError(e.to_string()));
            }
        };

        Ok(message)
    }

    fn process_sse_chunks<F>(
        &self,
        inner: LuaTable,
        chunk_receiver: Option<F>,
        resp: reqwest::blocking::Response,
    ) -> TinyResult<AssistantMessage>
    where
        F: Fn(Chunk) -> TinyResult<()>,
    {
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

                        let chunk_table =
                            inner.call_method::<LuaValue>("process_chunk", chunk_str)?;
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
                            chunk_rc(chunk)?;
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
