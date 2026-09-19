use std::collections::BTreeMap;

use serde_json::{Value as JsonValue, json};

use crate::core::{
    ContentPart, ImageDetail, Message, ThinkingOptions, ThinkingType, Tool, ToolCall,
    ToolParameter, UserMessage,
};

#[inline]
pub(super) fn tool_call_to_value(tool_call: &ToolCall) -> JsonValue {
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
pub(super) fn part_to_value(part: &ContentPart) -> JsonValue {
    match part {
        ContentPart::File(file) => {
            json!({
                "type": "file_url",
                "file_url": {
                    "url": file
                }
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
                "type": "image_url",
                "image_url": {
                    "url": url,
                    "detail": detail_str
                }
            })
        }
        ContentPart::Video(video) => {
            json!({
                "type": "video_url",
                "video_url": {
                    "url": video
                }
            })
        }
    }
}

pub(super) fn message_to_json_value(message: &Message) -> JsonValue {
    match message {
        Message::System(prompt) => {
            json!({
                "role": "system",
                "content": prompt
            })
        }
        Message::Tool {
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
        Message::User(msg) => match msg {
            UserMessage::Text(text) => {
                json!({
                    "role": "user",
                    "content": text
                })
            }
            UserMessage::Parts(parts) => {
                let content: Vec<JsonValue> = parts.iter().map(|p| part_to_value(p)).collect();

                json!({
                    "role": "user",
                    "content": content
                })
            }
        },
        Message::Assistant {
            content,
            reasoning_content,
            reasoning_details,
            tool_calls,
            finished_reason: _,
        } => {
            let tool_call_value = if let Some(tool_calls) = tool_calls {
                JsonValue::Array(
                    tool_calls
                        .iter()
                        .map(|t| tool_call_to_value(t))
                        .collect::<Vec<JsonValue>>(),
                )
            } else {
                JsonValue::Null
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
pub(super) fn build_thinking_option(options: &ThinkingOptions) -> JsonValue {
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

#[inline]
pub(super) fn tool_parameter_to_json_value(tool_param: &ToolParameter) -> JsonValue {
    json!({
        "type": &tool_param.p_type,
        "description": &tool_param.description
    })
}

#[inline]
pub(super) fn tool_to_json_value(tool: &Tool) -> JsonValue {
    let parameters = if tool.parameters.len() > 0 {
        let mut map = BTreeMap::new();

        for p in tool.parameters.iter() {
            map.insert(p.name.clone(), tool_parameter_to_json_value(p));
        }

        serde_json::to_value(map).unwrap()
    } else {
        JsonValue::Null
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
