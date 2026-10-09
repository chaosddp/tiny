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

impl FromLua for UserContentPart {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        let LuaValue::Table(table) = value else {
            return Err(mlua::Error::UserDataTypeMismatch);
        };

        let p_type: String = table.get("type")?;

        match p_type.as_ref() {
            "text" => Ok(UserContentPart::Text(table.get("text")?)),
            "file" => Ok(UserContentPart::File(table.get("file")?)),
            "video" => Ok(UserContentPart::Video(table.get("video")?)),
            "image" => Ok(UserContentPart::Image {
                image: table.get("image")?,
                detail: table.get("detail").unwrap_or(ImageDetail::Auto),
            }),
            _ => Err(mlua::Error::UserDataTypeMismatch),
        }
    }
}

///
/// generate following lua table:
///
/// 1. text
/// {
///     type = "text",
///     text = "text part"
/// }
///
/// 2. file
/// {
///     type = "file",
///     file = "file_url"
/// }
///
/// 3. video
/// {
///     type = "video",
///     video = "video_url"
/// }
///
/// 3. image
/// {
///     type = "image",
///     image = "image data url",
///     detail = "auto"
/// }
///
impl IntoLua for &UserContentPart {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;

        match self {
            UserContentPart::Text(t) => {
                table.set("type", "text")?;
                table.set("text", t.to_string())?;
            }
            UserContentPart::File(f) => {
                table.set("type", "file")?;
                table.set("file", f.to_string())?;
            }
            UserContentPart::Video(v) => {
                table.set("type", "video")?;
                table.set("video", v.to_string())?;
            }
            UserContentPart::Image { image, detail } => {
                table.set("type", "image")?;
                table.set("image", image.to_string())?;
                table.set("detail", detail.clone())?;
            }
        }

        Ok(LuaValue::Table(table))
    }
}

///
/// generate following lua table
///
/// 1. text
/// {
///     content = "text content"
/// }
///
/// 2. parts
/// {
///     parts = {
///         {
///             type = "text",
///             text = "text content"
///         },
///         {
///             type = "image",
///             image = "data:image/jpg,base64;...",
///         }
///     }
/// }
impl IntoLua for &UserMessage {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;

        match self {
            UserMessage::Text(t) => {
                table.set("content", t.to_string())?;
            }
            UserMessage::Parts(parts) => {
                let parts_table = lua.create_table()?;

                for part in parts {
                    parts_table.push(part)?;
                }

                table.set("parts", parts_table)?;
            }
        }

        Ok(LuaValue::Table(table))
    }
}

impl FromLua for UserMessage {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Table(table) => {
                if let Some(content) = table.get::<Option<String>>("content")? {
                    Ok(UserMessage::Text(content))
                } else {
                    Ok(UserMessage::Parts(table.get("parts")?))
                }
            }
            _ => Err(mlua::Error::UserDataTypeMismatch),
        }
    }
}

///
/// generate following lua table
///
/// 1. system
/// {
///     role = "system",
///     content = "system prompt"
/// }
///
/// 2. assistant
/// {
///     role = "assistant",
///     content = "content",
///     reasoning = "reasoning content",
///     reasoning_details = {"detail1"},
///     finish_reason = "stop",
///     usages = {
///         prompt: 0,
///         completion: 0,
///         total: 0
///     }
/// }
///
/// 3. tool
/// {
///     role = "tool",
///     name = "my_tool_name",
///     cotnent = "tool result"
/// }
///
/// 4. user
/// {
///     role = "user",
///     content = "content", --optional, content or parts
///     parts = {
///         ...
///     }
/// }
impl IntoLua for &ChatMessage {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        match self {
            ChatMessage::System(prompt) => {
                let table = lua.create_table()?;

                table.set("role", "system")?;
                table.set("content", prompt.to_string())?;

                Ok(LuaValue::Table(table))
            }
            ChatMessage::Assistant(msg) => {
                let table = msg.clone().into_lua(lua)?;

                table.as_table().unwrap().set("role", "assistant")?;

                Ok(table)
            }
            ChatMessage::Tool(msg) => {
                let table = msg.clone().into_lua(lua)?;

                table.as_table().unwrap().set("role", "tool")?;

                Ok(table)
            }
            ChatMessage::User(msg) => {
                let table = msg.into_lua(lua)?;

                table.as_table().unwrap().set("role", "user")?;

                Ok(table)
            }
        }
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
