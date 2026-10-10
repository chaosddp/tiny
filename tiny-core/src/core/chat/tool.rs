use mlua::IntoLua;
use serde::{Deserialize, Serialize};
use tiny_macros::{FromLuaTable, IntoLuaTable};

#[derive(Debug, Deserialize, Serialize, Clone, FromLuaTable, IntoLuaTable)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub r#type: String,
    pub required: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, FromLuaTable, IntoLuaTable)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Option<Vec<ToolParameter>>,
}
