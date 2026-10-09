use super::messages::{FinishReason, ToolCall};
use mlua::{FromLua, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Chunk {
    pub id: String,
    pub content: Option<String>,
    pub reasoning: Option<String>,
    pub tool_call: Option<ToolCall>,
    pub finish_reason: Option<FinishReason>,
    // pub done: bool,
}

impl FromLua for Chunk {
    fn from_lua(
        value: mlua::prelude::LuaValue,
        _: &mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        match value {
            LuaValue::Table(t) => Ok(Self {
                id: t.get("id")?,
                content: t.get("content")?,
                reasoning: t.get("reasoning")?,
                tool_call: t.get("tool_call")?,
                finish_reason: t.get("finish_reason")?,
            }),
            _ => Err(mlua::Error::UserDataTypeMismatch),
        }
    }
}
