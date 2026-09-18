#[cfg(feature = "async")]
pub mod async_impl;

#[cfg(feature = "sync")]
pub mod sync_impl;

#[cfg(feature = "async")]
use futures_util::StreamExt;
#[cfg(feature = "async")]
use reqwest::header;
#[cfg(feature = "async")]
use tokio::sync::mpsc;

use log::debug;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{
    Value::{self, Null},
    json, to_value,
};

use crate::core::{Tool, ToolParameter};

use super::core::{
    ChatOptions, ContentPart, FinishReason, ImageDetail, Message, MessageChunk, ReasoningEffort,
    ThinkingOptions, ThinkingType, TinyError, ToolCall, UserMessage,
};

#[derive(Debug, Serialize, Deserialize)]
struct OpenaiToolFunction {
    name: String,
    arguments: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenaiToolCall {
    id: String,
    index: u32,
    r#type: String,
    function: OpenaiToolFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenaiChoiceDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub reasoning: Option<String>,
    pub tool_calls: Option<Vec<OpenaiToolCall>>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenaiChoice {
    pub index: u32,
    pub delta: OpenaiChoiceDelta,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenaiChunk {
    id: String,
    object: String,
    created: u64,
    model: String,
    system_fingerprint: String,
    choices: Vec<OpenaiChoice>,
}

#[inline]
fn tool_call_to_value(tool_call: &ToolCall) -> Value {
    json!({
        "id": tool_call.id,
        "type": "function",
        "function": {
            "name": tool_call.name,
            "arguments": &tool_call.arguments
        }
    })
}

#[inline]
fn part_to_value(part: &ContentPart) -> Value {
    match part {
        ContentPart::File(file) => {
            json!({
                "type": "file",
                "file_url": file
            })
        }
        ContentPart::Text(text) => {
            json!({
                "type": "text",
                "text": text
            })
        }
        ContentPart::Image { url, detail } => {
            let detail_str = match detail {
                ImageDetail::Auto => "auto",
                ImageDetail::High => "high",
                ImageDetail::Low => "low",
                ImageDetail::Other(d) => d,
            };

            json!({
                "type": "image",
                "image_url": {
                    "url": url,
                    "detail": detail_str
                }
            })
        }
        ContentPart::Video(video) => {
            json!({
                "type": "video",
                "video_url": video
            })
        }
    }
}

fn message_to_json_value(message: &Message) -> serde_json::Value {
    match message {
        Message::SystemMessage(prompt) => {
            json!({
                "role": "system",
                "content": prompt
            })
        }
        Message::ToolMessage {
            content,
            tool_call_id,
            name,
        } => {
            json!({
                "role": "tool",
                "name": name,
                "tool_call_id": tool_call_id,
                "content": content
            })
        }
        Message::UserMessage(msg) => match msg {
            UserMessage::Text(text) => {
                json!({
                    "role": "user",
                    "content": text
                })
            }
            UserMessage::Parts(parts) => {
                let content: Vec<Value> = parts.iter().map(|p| part_to_value(p)).collect();

                json!({
                    "role": "user",
                    "content": content
                })
            }
        },
        Message::AssistantMessage {
            content,
            reasoning_content,
            reasoning_details,
            tool_calls,
            finished_reason: _,
        } => {
            let tool_call_value = if let Some(tool_calls) = tool_calls {
                Value::Array(
                    tool_calls
                        .iter()
                        .map(|t| tool_call_to_value(t))
                        .collect::<Vec<Value>>(),
                )
            } else {
                Value::Null
            };

            json!({
                "role": "assistant",
                "content": &content,
                "reasoning_cotnent": &reasoning_content,
                "reasoning_details": &reasoning_details,
                "tool_calls": tool_call_value
            })
        }
    }
}

#[inline]
fn build_thinking_option(options: &ThinkingOptions) -> Value {
    return json!({
        "type": match &options.t_type {
            ThinkingType::Enabled=>"enabled",
            ThinkingType::Disabled=>"disabled",
            ThinkingType::Adaptive=>"adaptive",
            ThinkingType::Other(s)=>s,
        },
        "budget_tokens": &options.budget_tokens
    });
}

fn tool_parameter_to_json_value(tool_param: &ToolParameter) -> Value {
    json!({
        "type": &tool_param.p_type,
        "description": &tool_param.description
    })
}

fn tool_to_json_value(tool: &Tool) -> Value {
    let parameters = if tool.parameters.len() > 0 {
        let mut map = BTreeMap::new();

        for p in tool.parameters.iter() {
            map.insert(p.name.clone(), tool_parameter_to_json_value(p));
        }

        to_value(map).unwrap()
    } else {
        Null
    };

    let required_parameters: Vec<String> = tool
        .parameters
        .iter()
        .filter(|p| p.required)
        .map(|p| p.name.clone())
        .collect();

    json!({
        "type": "function",
        "function": {
            "name": &tool.name,
            "description": &tool.description,
            "parameters": {
                "type": "object",
                "properties": parameters
            },
            "required": required_parameters
        }
    })
}
