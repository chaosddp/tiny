pub async fn chat(
    options: &ChatOptions,
    messages: &Vec<Message>,
    tools: &Vec<Tool>,
    chunk_sender: mpsc::Sender<MessageChunk>,
) -> Result<Message, TinyError> {
    let message_value_list: Vec<Value> =
        messages.iter().map(|m| message_to_json_value(m)).collect();

    let tool_definitions: Vec<Value> = tools.iter().map(|t| tool_to_json_value(t)).collect();

    let payload = json!({
            "model": &options.model,
            "base_url": &options.base_url,
            "stream": options.stream,
            "max_tokens": options.max_token,
            "thinking": if let Some(to) = &options.thinking {
                build_thinking_option(&to)
            } else {Null},
            "reasoning_effort": if let Some(re) = &options.reasoning_effort {
                json!(match re {
                    ReasoningEffort::Low=>"low",
                    ReasoningEffort::Medium=>"medium",
                    ReasoningEffort::High=>"high",
                    ReasoningEffort::Other(s)=>s
                })
            } else {Null},
            "include_usage": options.include_usage,
            "messages": message_value_list,
            "tools": tool_definitions
    });

    let mut headers = header::HeaderMap::new();

    headers.insert(
        "Authorization",
        format!("Bearer {}", &options.api_key).parse().unwrap(),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|_e| TinyError::RuntimeError)?;

    let chat_url = format!("{}/chat/completions", &options.base_url);

    let mut resp = client
        .post(chat_url)
        .json(&payload)
        .send()
        .await
        .map_err(|_e| TinyError::RuntimeError)?
        .bytes_stream();

    let mut content_builder = String::new();
    let mut reasoning_builder = String::new();
    let mut tool_calls: Vec<ToolCall> = vec![];
    let mut finish_reason = None;

    while let Some(chunk) = resp.next().await {
        match chunk {
            Ok(bytes) => {
                let content_str = String::from_utf8_lossy(&bytes).to_string();
                let content_parts: Vec<&str> = content_str.trim().split("\n\n").collect();

                debug!("recive raw chunk: {}", content_str);

                for content_part in content_parts {
                    let content_part = content_part
                        .strip_prefix("data: ")
                        .unwrap_or_default()
                        .trim();

                    if content_part == "[DONE]" {
                        break;
                    } else {
                        if let Ok(c) = serde_json::from_str::<OpenaiChunk>(content_part) {
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
                                chunk_sender
                                    .send(MessageChunk::Chunk {
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
                                    })
                                    .await
                                    .map_err(|_e| TinyError::RuntimeError)?;
                            }
                        }
                    }
                }
            }
            Err(_e) => {}
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
        tool_calls: if tool_calls.len() > 0 {
            Some(tool_calls)
        } else {
            None
        },
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
