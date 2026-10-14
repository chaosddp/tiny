use crate::{
    TinyResult,
    agent::event_hub::EventHub,
    core::chat::messages::{AssistantMessage, ChatMessage},
};
use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use tiny_macros::IntoLuaTable;

#[derive(Debug, Serialize, Deserialize, Clone, FromLua, IntoLuaTable)]
pub struct ChatOptions {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub stream: Option<bool>,
    pub stream_include_usage: Option<bool>,
    pub max_tokens: Option<usize>,
    pub reasoning_effort: Option<String>,
}

pub struct ChatClient {
    inner: LuaTable,
}

impl ChatClient {
    pub fn new(inner: LuaTable) -> Self {
        Self { inner }
    }

    pub fn chat<F, FO>(
        &self,
        client: &str,
        messages: &Vec<ChatMessage>,
        options: &ChatOptions,
        event_hub: &EventHub,
    ) -> TinyResult<AssistantMessage>
    where
        F: Fn(&LuaValue) -> bool,
        FO: FnOnce(&LuaValue) -> bool,
    {
        todo!()
    }
}
