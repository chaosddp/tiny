use super::messages::{FinishReason, ToolCall};
use serde::{Deserialize, Serialize};
use tiny_macros::{FromLuaTable, IntoLuaTable};

#[derive(Debug, Deserialize, Serialize, Clone, FromLuaTable, IntoLuaTable)]
pub struct Chunk {
    pub id: String,
    pub content: Option<String>,
    pub reasoning: Option<String>,
    pub tool_call: Option<ToolCall>,
    pub finish_reason: Option<FinishReason>,
}
