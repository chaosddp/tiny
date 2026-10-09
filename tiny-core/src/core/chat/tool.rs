use mlua::{FromLua, IntoLua, Lua, prelude::*};
use serde::{Deserialize, Serialize};
use tiny_macros::IntoLuaTable;

#[derive(Debug, Deserialize, Serialize, Clone, FromLua, IntoLuaTable)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub r#type: String,
    pub required: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, FromLua, IntoLuaTable)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Option<Vec<ToolParameter>>,
}
