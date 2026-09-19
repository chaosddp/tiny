use std::io::{BufRead, BufReader};

use log::debug;
use reqwest::{blocking, header};
use serde_json::{Value as JsonValue, json};

use crate::{
    core::{
        ChatClient, ChatOptions, FinishReason, Message, MessageChunk, ReasoningEffort, TinyError,
        Tool, ToolCall,
    },
    openai::{
        types::OpenAIChunk,
        utils::{build_thinking_option, message_to_json_value, tool_to_json_value},
    },
};

/// OpenAI compatible chat client
pub struct OpenaiClient {}

impl OpenaiClient {
    pub fn new() -> Self {
        OpenaiClient {}
    }
}

impl ChatClient for OpenaiClient {
    fn chat(
        &self,
        options: &ChatOptions,
        messages: &Vec<Message>,
        tools: &Vec<Tool>,
        chunk_receiver: &Box<dyn crate::core::sync_impl::ChunkReceiver>,
    ) -> Result<Message, TinyError> {
        let message_value_list: Vec<JsonValue> =
            messages.iter().map(|m| message_to_json_value(m)).collect();

        let tool_definitions: Vec<JsonValue> =
            tools.iter().map(|t| tool_to_json_value(t)).collect();

        let payload = json!({
                "model": &options.model,
                "base_url": &options.base_url,
                "stream": options.stream,
                "max_tokens": options.max_token,
                "thinking": if let Some(to) = &options.thinking {
                    build_thinking_option(&to)
                } else {JsonValue::Null},
                "reasoning_effort": if let Some(re) = &options.reasoning_effort {
                    json!(match re {
                        ReasoningEffort::Low=>"low",
                        ReasoningEffort::Medium=>"medium",
                        ReasoningEffort::High=>"high",
                        ReasoningEffort::Other(s)=>s
                    })
                } else {JsonValue::Null},
                "include_usage": options.include_usage,
                "messages": message_value_list,
                "tools": tool_definitions
        });

        debug!("payload: \n{:?}", payload);

        let mut headers = header::HeaderMap::new();

        headers.insert(
            "Authorization",
            format!("Bearer {}", &options.api_key).parse().unwrap(),
        );

        let client = blocking::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .map_err(|_e| TinyError::RuntimeError)?;

        let chat_url = format!("{}/chat/completions", &options.base_url);

        let resp = client
            .post(chat_url)
            .json(&payload)
            .send()
            .map_err(|_e| TinyError::RuntimeError)?;

        let mut content_builder = String::new();
        let mut reasoning_builder = String::new();
        let mut tool_calls: Vec<ToolCall> = vec![];
        let mut finish_reason = None;

        let reader = BufReader::new(resp);

        for line_ret in reader.lines() {
            match line_ret {
                Ok(chunk_str) => {
                    debug!("recive raw chunk: {}", chunk_str);

                    let content_part = chunk_str.strip_prefix("data: ").unwrap_or_default().trim();

                    if content_part == "[DONE]" {
                        break;
                    } else {
                        if let Ok(c) = serde_json::from_str::<OpenAIChunk>(content_part) {
                            let first_choice = c.choices.first().unwrap();

                            if let Some(content) = &first_choice.delta.content
                                && content.len() > 0
                            {
                                content_builder.push_str(&content.clone());
                            }

                            if let Some(reasoning) = &first_choice.delta.reasoning
                                && reasoning.len() > 0
                            {
                                reasoning_builder.push_str(&reasoning.clone());
                            }

                            if let Some(fr) = &first_choice.delta.finish_reason
                                && fr.len() > 0
                            {
                                finish_reason = Some(fr.to_string())
                            }

                            if let Some(tc_list) = &first_choice.delta.tool_calls {
                                for tc in tc_list {
                                    tool_calls.push(ToolCall {
                                        name: tc.function.name.clone(),
                                        id: tc.id.clone(),
                                        index: tc.index,
                                        arguments: tc.function.arguments.clone(),
                                    });
                                }
                            }

                            if finish_reason.is_none() {
                                chunk_receiver.chunk(MessageChunk::Chunk {
                                    content: if let Some(c) = &first_choice.delta.content
                                        && c.len() > 0
                                    {
                                        Some(c.to_string())
                                    } else {
                                        None
                                    },
                                    reasoning_content: if let Some(rc) =
                                        &first_choice.delta.reasoning
                                        && rc.len() > 0
                                    {
                                        Some(rc.to_string())
                                    } else {
                                        None
                                    },
                                    tool_calls: if let Some(tc_list) =
                                        &first_choice.delta.tool_calls
                                    {
                                        Some(
                                            tc_list
                                                .iter()
                                                .map(|tc| ToolCall {
                                                    name: tc.function.name.clone(),
                                                    id: tc.id.clone(),
                                                    index: tc.index,
                                                    arguments: tc.function.arguments.clone(),
                                                })
                                                .collect::<Vec<ToolCall>>(),
                                        )
                                    } else {
                                        None
                                    },
                                    tool_result: None,
                                })?;
                            }
                        }
                    }
                }
                Err(_e) => {}
            };
        }

        Ok(Message::Assistant {
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
            tool_calls: if tool_calls.len() > 0 {
                Some(tool_calls)
            } else {
                None
            },
            finished_reason: match finish_reason {
                Some(fr) => match fr.as_ref() {
                    "tool_call" | "tool_calls" => FinishReason::ToolCall,
                    "stop" => FinishReason::Stop,
                    "length" => FinishReason::Length,
                    "content_filter" => FinishReason::ContentFilter,
                    _ => FinishReason::Other(fr),
                },
                _ => FinishReason::Other("".to_string()),
            },
        })
    }
}
