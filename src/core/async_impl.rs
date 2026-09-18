

// short-cut of the async chat function type
pub trait ChatAsyncFn:
    AsyncFn(
    &ChatOptions,
    &Vec<Message>,
    &Vec<Tool>,
    mpsc::Sender<MessageChunk>,
) -> Result<Message, TinyError>
{
}

// implement it for all matching functions
impl<F> ChatAsyncFn for F where
    F: AsyncFn(
        &ChatOptions,
        &Vec<Message>,
        &Vec<Tool>,
        mpsc::Sender<MessageChunk>,
    ) -> Result<Message, TinyError>
{
}

pub trait ToolAsyncFn: AsyncFn(&str, &str, Option<&str>) -> Result<String, TinyError> {}

impl<F> ToolAsyncFn for F where F: AsyncFn(&str, &str, Option<&str>) -> Result<String, TinyError> {}

pub async fn tiny_loop<C>(
    options: &ChatOptions,
    mut messages: Vec<Message>,
    chat: C,
    tools: &Vec<Tool>,
    mut tool_execute: (
        mpsc::Sender<(String, String, Option<String>)>,
        mpsc::Receiver<String>,
    ),
    chunk_sender: mpsc::Sender<MessageChunk>,
) -> Result<Vec<Message>, TinyError>
where
    C: ChatAsyncFn,
{
    loop {
        let msg = chat(options, &messages, tools, chunk_sender.clone()).await?;

        if let Message::AssistantMessage {
            content: _,
            reasoning_content: _,
            reasoning_details: _,
            tool_calls,
            finished_reason,
        } = &msg
        {
            if *finished_reason == FinishReason::ToolCall || tool_calls.is_none() {
                messages.push(msg);

                break;
            }

            if let Some(tool_calls) = tool_calls {
                for tool_call in tool_calls {
                    // let tool_call_ret = tool_execute(
                    //     &tool_call.name,
                    //     &tool_call.id,
                    //     tool_call.arguments.as_deref(),
                    // )
                    // .await?;
                    debug!(
                        "Start tool call ({}): {}({:?})",
                        tool_call.id, tool_call.name, tool_call.arguments
                    );
                    tool_execute
                        .0
                        .send((
                            tool_call.name.to_string(),
                            tool_call.id.to_string(),
                            tool_call.arguments.clone(),
                        ))
                        .await
                        .map_err(|e| TinyError::RuntimeError)?;

                    if let Some(ret) = tool_execute.1.recv().await {
                        messages.push(Message::ToolMessage {
                            content: ret,
                            tool_call_id: tool_call.id.clone(),
                            name: tool_call.name.clone(),
                        });
                    } else {
                        messages.push(Message::ToolMessage {
                            content: format!("Cannot get result from tool: {}", tool_call.name),
                            tool_call_id: tool_call.id.clone(),
                            name: tool_call.name.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(messages)
}
