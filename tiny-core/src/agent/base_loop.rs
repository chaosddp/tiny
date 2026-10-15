use crate::{
    TinyResult,
    agent::{
        chat_client::{ChatClient, ChatOptions},
        context::Context,
    },
    core::chat::{
        chunk::Chunk,
        messages::{ChatMessage, FinishReason, ToolMessage},
        tool::Tool,
    },
};
use mlua::prelude::*;

pub fn base_loop<F>(
    lua: &Lua,
    client: &str,
    messages: &mut Vec<ChatMessage>,
    options: &ChatOptions,
    chat_client: &ChatClient,
    tools: Option<&Vec<Tool>>,
    chunk_receiver: F,
    context: &Context,
) -> TinyResult<()>
where
    F: Fn(Chunk) -> TinyResult<()>,
{
    context.event_hub.send("loop_start", LuaValue::Nil);

    let mut n = 0;
    loop {
        n += 1;

        context.event_hub.send("loop_round", n);

        // TODO: try to fetch content that need to insert into the message list, like user input while waiting for model response

        // TODO: error handling: call chat fail strategy with and response, retry? cancel? patching?
        let message = chat_client.chat(client, messages, tools, options, &chunk_receiver)?;

        context.event_hub.send(
            "message",
            ChatMessage::Assistant(message.clone()).into_lua(lua)?,
        );

        let orig_size = messages.len();

        if message.finish_reason == FinishReason::ToolCall
            && let Some(tool_calls) = &message.tool_calls
        {
            if tool_calls.len() > 0 {
                for tool_call in tool_calls {
                    // TODO: error hanling: retry? cancel?
                    let tool_result = context
                        .tool_executor
                        .execute(&tool_call.name, tool_call.arguments.as_deref())?;

                    let tool_message = ChatMessage::Tool(ToolMessage {
                        id: tool_call.id.to_string(),
                        name: tool_call.name.to_string(),
                        content: tool_result,
                    });

                    context
                        .event_hub
                        .send("message", (&tool_message).into_lua(lua)?);

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

    context.event_hub.send("loop_end", LuaValue::Nil);

    Ok(())
}
