use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use tiny_macros::IntoLuaTable;

use crate::core::{
    AssistantMessage, ChatMessage, Chunk, FinishReason, TinyResult, Tool, ToolMessage,
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
    fn emit<T>(&self, event_name: &str, args: &T) -> TinyResult<()>
    where
        T: IntoLua + Clone;
}

pub fn base_loop<CC, CR, TE>(
    messages: &mut Vec<ChatMessage>,
    options: &ChatOptions,
    chat_client: CC,
    tools: &Vec<Tool>,
    tool_executor: TE,
    chunk_receiver: CR,
    handlers: &impl EventHandler,
) -> TinyResult<()>
where
    CC: Fn(&Vec<ChatMessage>, &Vec<Tool>, &ChatOptions, &CR) -> TinyResult<AssistantMessage>,
    CR: Fn(Chunk) -> TinyResult<()>,
    TE: Fn(String, Option<String>) -> TinyResult<String>,
{
    handlers.emit("event_name", &LuaValue::Nil)?;

    let mut n = 0;
    loop {
        n += 1;

        handlers.emit("loop_round", &n)?;

        // TODO: try to fetch content that need to insert into the message list, like user input while waiting for model response

        // TODO: error handling: call chat fail strategy with and response, retry? cancel? patching?
        let message = chat_client(messages, tools, options, &chunk_receiver)?;

        handlers.emit("message", &&ChatMessage::Assistant(message.clone()))?;

        let orig_size = messages.len();

        if message.finish_reason == FinishReason::ToolCall
            && let Some(tool_calls) = &message.tool_calls
        {
            if tool_calls.len() > 0 {
                for tool_call in tool_calls {
                    // TODO: error hanling: retry? cancel?
                    let tool_result =
                        tool_executor(tool_call.name.to_string(), tool_call.arguments.clone())?;

                    let tool_message = ToolMessage {
                        id: tool_call.id.to_string(),
                        name: tool_call.name.to_string(),
                        content: tool_result,
                    };

                    handlers.emit("message", &&ChatMessage::Tool(tool_message.clone()))?;

                    messages.push(ChatMessage::Tool(tool_message));
                }
            }
        }

        messages.insert(orig_size, ChatMessage::Assistant(message));

        // if the assistant message is the only one we got, then break.
        if messages.len() - orig_size == 1 {
            break;
        }
    }

    handlers.emit("loop_end", &LuaValue::Nil)?;

    Ok(())
}
