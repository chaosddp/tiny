use std::sync::{Arc, MutexGuard};

use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::mpsc;

use crate::core::common::{
    ChatClient, ChatOptions, ContentPart, FinishReason, ImageDetail, Message, MessageChunk,
    TinyError, ToolCall, UserMessage,
};

#[derive(Serialize, Deserialize)]
pub struct ChoiceDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub reasoning: Option<String>,
    pub finish_reason: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub delta: ChoiceDelta,
}

#[derive(Serialize, Deserialize)]
pub struct Chunk {
    id: String,
    object: String,
    created: u64,
    model: String,
    system_fingerprint: String,
    choices: Vec<Choice>,
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
            UserMessage::Rich(parts) => {
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

pub struct OpenaiChatClient {}

impl Default for OpenaiChatClient {
    fn default() -> Self {
        Self {}
    }
}

#[async_trait]
impl ChatClient for OpenaiChatClient {
    async fn chat(
        &self,
        options: Arc<ChatOptions>,
        messages: &Vec<Message>,
        chunk_sender: mpsc::Sender<MessageChunk>,
    ) -> Result<Message, TinyError> {
        let message_value_list: Vec<Value> =
            messages.iter().map(|m| message_to_json_value(m)).collect();

        let payload = json!({
                "model": &options.model,
                "base_url": &options.base_url,
                "stream": options.stream,
                "max_tokens": options.max_token,
                "thinking": {
                    "type": "enabled"
                },
                "messages": message_value_list
        });

        let mut headers = header::HeaderMap::new();

        headers.insert(
            "Authorization",
            format!("Bearer {}", &options.api_key).parse().unwrap(),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| TinyError::RuntimeError)?;

        let chat_url = format!("{}/chat/completions", &options.base_url);

        let mut resp = client
            .post(chat_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| TinyError::RuntimeError)?
            .bytes_stream();

        let mut content_builder = String::new();
        let mut reasoning_builder = String::new();
        let mut finish_reason = None;

        while let Some(chunk) = resp.next().await {
            match chunk {
                Ok(bytes) => {
                    let content_str = String::from_utf8_lossy(&bytes).to_string();
                    let content_parts: Vec<&str> = content_str.trim().split("\n\n").collect();

                    // println!("{}", content_str);

                    for content_part in content_parts {
                        let content_part = content_part
                            .strip_prefix("data: ")
                            .unwrap_or_default()
                            .trim();

                        // println!("{}", content_part);

                        if content_part == "[DONE]" {
                            break;
                        } else {
                            if let Ok(c) = serde_json::from_str::<Chunk>(content_part) {
                                let first_choice = c.choices.first().unwrap();

                                if let Some(content) = &first_choice.delta.content {
                                    content_builder.push_str(&content.clone());
                                }

                                if let Some(reasoning) = &first_choice.delta.reasoning {
                                    reasoning_builder.push_str(&reasoning.clone());
                                }

                                if let Some(fr) = &first_choice.delta.finish_reason {
                                    finish_reason = Some(fr.to_string())
                                }

                                if finish_reason.is_none() {
                                    chunk_sender
                                        .send(MessageChunk::Chunk {
                                            content: first_choice.delta.content.clone(),
                                            reasoning_content: first_choice.delta.reasoning.clone(),
                                            tool_calls: None,
                                        })
                                        .await
                                        .map_err(|e| TinyError::RuntimeError)?;
                                }
                            }
                        }
                    }
                }
                Err(e) => {}
            };
        }

        Ok(Message::AssistantMessage {
            content: if content_builder.len() > 0 {
                Some(content_builder)
            } else {
                None
            },
            reasoning_content: if reasoning_builder.len() > 0 {
                Some(reasoning_builder)
            } else {
                None
            },
            reasoning_details: None,
            tool_calls: None,
            finished_reason: match finish_reason {
                Some(fr) => {
                    if fr == "tool_call" {
                        FinishReason::ToolCall
                    } else {
                        FinishReason::Other(fr)
                    }
                }
                _ => FinishReason::Other("".to_string()),
            },
        })
    }
}
