/*! This module container synchronization base agent loop.
 *
 * This implementation use trait(s) for callbacks and lifetime management
 */

use log::debug;

use super::{ChatOptions, FinishReason, Message, MessageChunk, TinyError, Tool};

/// Trait to receive stream chunk from api server.
pub trait ChunkReceiver {
    /// called on chunk arrived
    fn chunk(&self, chunk: MessageChunk) -> Result<(), TinyError>;
}

/// Trait to provide chat implementation for different providers.
pub trait ChatClient {
    /// called when the agent need to interactive with LLM
    fn chat(
        &self,
        options: &ChatOptions,
        messages: &Vec<Message>,
        tools: &Vec<Tool>,
        chunk_receiver: &Box<dyn ChunkReceiver>,
    ) -> Result<Message, TinyError>;
}

/// Trait to execute tool calls.
pub trait ToolExecutor {
    fn exec(&self, name: &str, id: &str, tool_args: Option<&str>) -> Result<String, TinyError>;
}

/// Basic agent loop
///
/// # Example
///
/// ```ignore
///
/// // your chat options
/// let options = ChatOptions{...};
///
/// let message: Vec<Message> = vec![];
///
/// let tools = load_my_tools();
///
/// let tool_executor: Box<dyn ToolExecutor> = MyToolExecutor::new();
///
/// let chat_client: Box<dyn ChatClient> = MyChatClient::new();
///
/// let chunk_reciever: Box<dyn ChunkReceiver> = MyChunkReceiver::new();
///
/// // one chat loop
/// tiny_loop(
///     &options,
///     &mut messages,
///     &chat_client,
///     &tools,
///     &tool_executor,
///     &chunk_receiver,
/// )?;
///
/// ```
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

        if let Message::Assistant {
            content: _,
            reasoning_content: _,
            reasoning_details: _,
            tool_calls,
            finished_reason,
        } = &msg
        {
            if *finished_reason != FinishReason::ToolCall || tool_calls.is_none() {
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

                    chunk_receiver.chunk(MessageChunk::Chunk {
                        content: None,
                        reasoning_content: None,
                        tool_calls: None,
                        tool_result: Some(tool_call_ret.to_string()),
                    })?;

                    messages.push(Message::Tool {
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
