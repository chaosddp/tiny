use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use tiny_macros::IntoLuaTable;

use crate::core::{
    TinyResult,
    chat::{
        chunk::Chunk,
        messages::{AssistantMessage, ChatMessage, FinishReason, ToolMessage, UserMessage},
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

pub trait EventHandler {
    fn loop_start(&self) -> TinyResult<()> {
        Ok(())
    }

    fn loop_end(&self) -> TinyResult<()> {
        Ok(())
    }

    fn loop_round(&self, _n: u32) -> TinyResult<()> {
        Ok(())
    }

    fn message(&self, _message: &ChatMessage) -> TinyResult<()> {
        Ok(())
    }

    fn assistant_message(&self, _message: &AssistantMessage) -> TinyResult<()> {
        Ok(())
    }

    fn user_message(&self, _message: &UserMessage) -> TinyResult<()> {
        Ok(())
    }
}

pub fn base_loop<CC, CR, TE>(
    messages: &mut Vec<ChatMessage>,
    options: &ChatOptions,
    chat_client: CC,
    tools: &Vec<Tool>,
    tool_executor: TE,
    chunk_receiver: CR,
    handlers: &Box<dyn EventHandler>,
) -> TinyResult<()>
where
    CC: Fn(&Vec<ChatMessage>, &Vec<Tool>, &ChatOptions, &CR) -> TinyResult<AssistantMessage>,
    CR: Fn(Chunk) -> TinyResult<()>,
    TE: Fn(String, Option<String>) -> TinyResult<String>,
{
    handlers.loop_start()?;

    let mut n = 0;
    loop {
        n += 1;

        handlers.loop_round(n)?;

        // TODO: try to fetch content that need to insert into the message list, like user input while waiting for model response

        // TODO: error handling: call chat fail strategy with and response, retry? cancel? patching?
        let message = chat_client(messages, tools, options, &chunk_receiver)?;

        handlers.assistant_message(&message)?;

        let orig_size = messages.len();

        if message.finish_reason == FinishReason::ToolCall
            && let Some(tool_calls) = &message.tool_calls
        {
            if tool_calls.len() > 0 {
                for tool_call in tool_calls {
                    // TODO: error hanling: retry? cancel?
                    let tool_result =
                        tool_executor(tool_call.name.to_string(), tool_call.arguments.clone())?;

                    let tool_message = ChatMessage::Tool(ToolMessage {
                        id: tool_call.id.to_string(),
                        name: tool_call.name.to_string(),
                        content: tool_result,
                    });

                    handlers.message(&tool_message)?;

                    messages.push(tool_message);
                }
            }
        }

        messages.insert(orig_size, ChatMessage::Assistant(message));

        // if the assistant message is the only one we got, then break.
        if messages.len() - orig_size == 1 {
            break;
        }
    }

    handlers.loop_end()?;

    Ok(())
}
