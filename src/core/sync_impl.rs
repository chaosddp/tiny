use log::debug;

use crate::core::{ChatOptions, FinishReason, Message, MessageChunk, TinyError, Tool};

// short-cut of the async chat function type
pub trait ChatFn:
    Fn(
    &ChatOptions,
    &Vec<Message>,
    &Vec<Tool>,
    fn(MessageChunk) -> Result<(), TinyError>,
) -> Result<Message, TinyError>
{
}

// implement it for all matching functions
impl<F> ChatFn for F where
    F: Fn(
        &ChatOptions,
        &Vec<Message>,
        &Vec<Tool>,
        fn(MessageChunk) -> Result<(), TinyError>,
    ) -> Result<Message, TinyError>
{
}

pub trait ToolExecuteFn: Fn(&str, &str, Option<&str>) -> Result<String, TinyError> {}

impl<F> ToolExecuteFn for F where F: Fn(&str, &str, Option<&str>) -> Result<String, TinyError> {}

pub fn tiny_loop<C, T>(
    options: &ChatOptions,
    mut messages: Vec<Message>,
    chat: C,
    tools: &Vec<Tool>,
    tool_execute: T,
    chunk_receiver: fn(MessageChunk) -> Result<(), TinyError>,
) -> Result<Vec<Message>, TinyError>
where
    C: ChatFn,
    T: ToolExecuteFn,
{
    loop {
        let msg = chat(options, &messages, tools, chunk_receiver)?;

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
                    debug!(
                        "Start tool call ({}): {}({:?})",
                        tool_call.id, tool_call.name, tool_call.arguments
                    );

                    let tool_call_ret = tool_execute(
                        &tool_call.name,
                        &tool_call.id,
                        tool_call.arguments.as_deref(),
                    )?;

                    messages.push(Message::ToolMessage {
                        content: tool_call_ret,
                        tool_call_id: tool_call.id.clone(),
                        name: tool_call.name.clone(),
                    });
                }
            }
        }
    }

    Ok(messages)
}
