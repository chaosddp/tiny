use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use tiny_macros::IntoLuaTable;

use crate::core::{
    TinyResult,
    chat::{
        chunk::Chunk,
        messages::{AssistantMessage, ChatMessage, FinishReason, ToolMessage},
        tool::Tool,
    },
};

#[derive(Debug, Serialize, Deserialize, Clone, FromLua, IntoLuaTable)]
pub struct ChatOptions {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub stream: Option<bool>,
    pub stream_include_usage: Option<bool>,
    pub max_tokens: Option<usize>,
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Default)]
pub struct BaseLoop();

pub struct LoopContext {}

impl BaseLoop {
    pub fn execute<CC, CR, TE>(
        &self,
        messages: &mut Vec<ChatMessage>,
        options: &ChatOptions,
        chat_client: CC,
        tools: Option<&Vec<Tool>>,
        tool_executor: Option<TE>,
        chunk_receiver: Option<CR>,
        _ctx: &LoopContext,
    ) -> TinyResult<()>
    where
        CC: Fn(
                &Vec<ChatMessage>,
                Option<&Vec<Tool>>,
                &ChatOptions,
                &Option<CR>,
            ) -> TinyResult<AssistantMessage>
            + 'static,
        CR: Fn(Chunk) -> TinyResult<()> + 'static,
        TE: Fn(&str, Option<&str>) -> TinyResult<String> + 'static,
    {
        // TODO: notify before loop start

        loop {
            // TODO: notify loop round N

            // TODO: try to fetch content that need to insert into the message list, like user input while waiting for model response

            // TODO: session update

            // TODO: error handling: call chat fail strategy with and response, retry? cancel? patching?
            let message = chat_client(messages, tools, options, &chunk_receiver)?;

            // TODO: notify about chat client response

            let orig_size = messages.len();

            if message.finish_reason == FinishReason::ToolCall
                && let Some(tool_exector) = &tool_executor
                && let Some(tool_calls) = &message.tool_calls
            {
                if tool_calls.len() > 0 {
                    // TODO: notify about tool call loop

                    for tool_call in tool_calls {
                        // TODO: notify about tool call detail

                        // TODO: error hanling: retry? cancel?
                        let tool_result =
                            tool_exector(&tool_call.name, tool_call.arguments.as_deref())?;

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
