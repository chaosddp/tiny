use crate::core::{
    TinyResult,
    chat::{
        chunk::Chunk,
        messages::{AssistantMessage, ChatMessage, FinishReason, ToolMessage},
    },
    decision::{DecisionMessage, DecisionResponse},
};

pub struct DecisionOptions {}

pub trait DecisionClient {
    fn make(
        &self,
        message: &DecisionMessage,
        options: &DecisionOptions,
    ) -> TinyResult<DecisionResponse>;
}

pub struct ChatOptions {}

pub trait ChunkReceiver {
    fn recv(&self, chunk: Chunk) -> TinyResult<()>;
}

pub trait ChatClient {
    fn chat(
        &self,
        messages: &Vec<ChatMessage>,
        options: &ChatOptions,
        chunk_receiver: Option<&Box<dyn ChunkReceiver>>,
    ) -> TinyResult<AssistantMessage>;
}

pub trait ToolExecutor {
    fn execute(&self, name: &str, arguments: Option<&str>) -> TinyResult<String>;
}

#[derive(Debug, Default)]
pub struct BaseLoop();

pub struct LoopContext {}

impl BaseLoop {
    pub fn execute(
        &self,
        messages: &mut Vec<ChatMessage>,
        options: &ChatOptions,
        chat_client: &Box<dyn ChatClient>,
        tool_executor: Option<&Box<dyn ToolExecutor>>,
        chunk_receiver: Option<&Box<dyn ChunkReceiver>>,
        _ctx: &LoopContext,
    ) -> TinyResult<()> {
        // TODO: notify before loop start

        loop {
            // TODO: notify loop round N

            // TODO: try to fetch content that need to insert into the message list, like user input while waiting for model response

            // TODO: session update

            // TODO: error handling: call chat fail strategy with and response, retry? cancel? patching?
            let message = chat_client.chat(messages, options, chunk_receiver)?;

            // TODO: notify about chat client response

            let orig_size = messages.len();

            if message.finish_reason == FinishReason::ToolCall
                && let Some(tool_exector) = tool_executor
                && let Some(tool_calls) = &message.tool_calls
            {
                if tool_calls.len() > 0 {
                    // TODO: notify about tool call loop

                    for tool_call in tool_calls {
                        // TODO: notify about tool call detail

                        // TODO: error hanling: retry? cancel?
                        let tool_result = tool_exector
                            .execute(&tool_call.name, tool_call.arguments.as_deref())?;

                        // TODO: notify about tool call result

                        messages.push(ChatMessage::Tool(ToolMessage {
                            id: tool_call.id.to_string(),
                            name: tool_call.name.to_string(),
                            content: tool_result,
                        }));
                    }
                }
            }

            messages.insert(orig_size, ChatMessage::Assistant(message));

            // if the assistant message is the only one we got, then break.
            if messages.len() - orig_size == 1 {
                break;
            }
        }

        // TODO: notify loop end

        Ok(())
    }
}
