use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use tiny_macros::IntoLuaTable;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ImageDetail {
    Auto,
    High,
    Low,
    Other(String),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCall,
    ContentFilter,
    Other(String),
}

#[derive(Debug, Deserialize, Serialize, Clone, FromLua, IntoLuaTable)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub index: usize,
    pub arguments: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, FromLua, IntoLuaTable)]
pub struct TokenUsage {
    pub prompt: usize,
    pub completion: usize,
    pub total: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone, FromLua, IntoLuaTable)]
pub struct ToolMessage {
    pub id: String,
    pub name: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, FromLua, IntoLuaTable)]
pub struct AssistantMessage {
    pub content: Option<String>,
    pub reasoning: Option<String>,
    pub reasoning_details: Option<Vec<String>>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub finish_reason: FinishReason,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Deserialize, Serialize, Clone, FromLua, IntoLuaTable)]
pub struct UserImageContentPart {
    pub image: String,
    pub detail: ImageDetail,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum UserContentPart {
    Text(String),
    Video(String),
    File(String),
    Image { image: String, detail: ImageDetail },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum UserMessage {
    Text(String),
    Parts(Vec<UserContentPart>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ChatMessage {
    System(String),
    User(UserMessage),
    Assistant(AssistantMessage),
    Tool(ToolMessage),
}

impl FromLua for ImageDetail {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::String(v) => {
                let s = v.to_str()?;

                Ok(match s.as_ref() {
                    "Auto" | "auto" => ImageDetail::Auto,
                    "Low" | "low" => ImageDetail::Low,
                    "High" | "high" => ImageDetail::High,
                    _ => ImageDetail::Other(s.to_string()),
                })
            }
            _ => Err(mlua::Error::UserDataTypeMismatch),
        }
    }
}

impl IntoLua for ImageDetail {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        Ok(LuaValue::String(lua.create_string(match self {
            ImageDetail::Auto => "auto".to_string(),
            ImageDetail::High => "high".to_string(),
            ImageDetail::Low => "low".to_string(),
            ImageDetail::Other(v) => v.to_string(),
        })?))
    }
}

impl FromLua for FinishReason {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::String(v) => {
                let s = v.to_str()?;

                Ok(match s.as_ref() {
                    "Stop" | "stop" => FinishReason::Stop,
                    "Length" | "length" => FinishReason::Length,
                    "ToolCall" | "tool_call" | "tool_calls" => FinishReason::ToolCall,
                    "ContentFilter" | "content_filter" => FinishReason::ContentFilter,
                    _ => FinishReason::Other(s.to_string()),
                })
            }
            _ => Err(mlua::Error::UserDataTypeMismatch),
        }
    }
}

impl IntoLua for FinishReason {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        Ok(LuaValue::String(lua.create_string(match self {
            FinishReason::Stop => "stop".to_string(),
            FinishReason::Length => "length".to_string(),
            FinishReason::ToolCall => "tool_call".to_string(),
            FinishReason::ContentFilter => "content_filter".to_string(),
            FinishReason::Other(s) => s.to_string(),
        })?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_message_serialize() {
        let lua = Lua::new();

        let msg: AssistantMessage = lua
            .from_value(
                lua.load(r#"{content = "content", reasoning="reasoning", finish_reason="Stop"}"#)
                    .eval()
                    .unwrap(),
            )
            .unwrap();

        assert_eq!(msg.content, Some("content".to_string()));
        assert_eq!(msg.reasoning, Some("reasoning".to_string()));
        assert!(msg.reasoning_details.is_none());
        assert!(msg.tool_calls.is_none());
        assert_eq!(msg.finish_reason, FinishReason::Stop);
        assert!(msg.usage.is_none());
    }

    #[test]
    fn test_struct_message_deserialize() {
        let lua = Lua::new();

        let msg = AssistantMessage {
            content: Some("content".to_string()),
            reasoning: Some("reasoning".to_string()),
            reasoning_details: None,
            tool_calls: Some(vec![ToolCall {
                id: "t1".to_string(),
                name: "my_tool".to_string(),
                index: 0,
                arguments: Some(r#"{"a": 1}"#.to_string()),
            }]),
            finish_reason: FinishReason::ToolCall,
            usage: None,
        };

        let table = msg.into_lua(&lua).unwrap();

        assert!(table.is_table());

        let table = table.as_table().unwrap();

        assert_eq!(
            table.get::<String>("content").unwrap(),
            "content".to_string()
        );
        assert_eq!(
            table.get::<String>("reasoning").unwrap(),
            "reasoning".to_string()
        );
        assert_eq!(
            table.get::<LuaValue>("reasoning_details").unwrap(),
            LuaValue::Nil
        );
    }
}
