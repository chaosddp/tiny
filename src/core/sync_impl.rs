use log::debug;

use crate::core::{ChatOptions, FinishReason, Message, MessageChunk, TinyError, Tool};

pub trait ChunkReceiver {
    fn chunk(&self, chunk: MessageChunk) -> Result<(), TinyError>;
}

pub trait ChatClient {
    fn chat(
        &self,
        options: &ChatOptions,
        messages: &Vec<Message>,
        tools: &Vec<Tool>,
        chunk_receiver: &Box<dyn ChunkReceiver>,
    ) -> Result<Message, TinyError>;
}

pub trait ToolExecutor {
    fn exec(&self, name: &str, id: &str, tool_args: Option<&str>) -> Result<String, TinyError>;
}

pub fn tiny_loop(
    options: &ChatOptions,
    messages: &mut Vec<Message>,
    chat_client: &Box<dyn ChatClient>,
    tools: &Vec<Tool>,
    tool_executor: &Box<dyn ToolExecutor>,
    chunk_receiver: &Box<dyn ChunkReceiver>,
) -> Result<(), TinyError> {
    loop {
        let msg = chat_client.chat(options, &messages, tools, chunk_receiver)?;

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

                    let tool_call_ret = tool_executor.exec(
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

    Ok(())
}
