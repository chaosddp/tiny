use std::sync::{Arc, Mutex, MutexGuard};

use async_trait::async_trait;
use serde_json::Value;

use thiserror::Error;

use tokio::sync::mpsc;

use crate::openai;

#[derive(Error, Debug)]
pub enum TinyError {
    #[error("runtime error")]
    RuntimeError,
}

#[derive(Debug)]
pub enum ImageDetail {
    Auto,
    Low,
    High,
    Other(String),
}

impl Default for ImageDetail {
    fn default() -> Self {
        ImageDetail::Auto
    }
}

#[derive(Debug)]
pub enum ContentPart {
    Text(String),
    Image { url: String, detail: ImageDetail },
    Video(String),
    File(String),
}

pub type RichContent = Vec<ContentPart>;

#[derive(Debug)]
pub enum UserMessage {
    Text(String),
    Rich(RichContent),
}

#[derive(Debug)]
pub struct ToolCall {
    pub name: String,
    pub id: String,
    pub arguments: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FinishReason {
    ToolCall,
    Other(String),
}

#[derive(Debug)]
pub enum Message {
    SystemMessage(String),
    UserMessage(UserMessage),
    AssistantMessage {
        content: Option<String>,
        reasoning_content: Option<String>,
        reasoning_details: Option<Vec<String>>,
        tool_calls: Option<Vec<ToolCall>>,
        finished_reason: FinishReason,
    },
    ToolMessage {
        content: String,
        tool_call_id: String,
        name: String,
    },
}

pub trait ToJsonValue {
    fn to_json_value(&self) -> Value;
}

pub struct ChatOptions {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub stream: bool,
    pub max_token: u32,
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self {
            model: std::env::var("TINY_DEFAULT_MODEL").unwrap_or(Default::default()),
            base_url: std::env::var("TINY_DEFAULT_BASE_URL").unwrap_or(Default::default()),
            api_key: std::env::var("TINY_DEFAULT_API_KEY").unwrap_or(Default::default()),
            stream: true,
            max_token: 8000,
        }
    }
}

pub enum MessageChunk {
    Chunk {
        content: Option<String>,
        reasoning_content: Option<String>,
        tool_calls: Option<Vec<ToolCall>>,
    },
    Error(String),
}

#[async_trait]
pub trait ChatClient: Default {
    async fn chat(
        &self,
        options: Arc<ChatOptions>,
        messages: &Vec<Message>,
        chunk_sender: mpsc::Sender<MessageChunk>,
    ) -> Result<Message, TinyError>;
}

pub async fn tiny_loop<C>(
    options: Arc<ChatOptions>,
    mut messages: Vec<Message>,
    chunk_sender: mpsc::Sender<MessageChunk>,
) -> Result<Vec<Message>, TinyError> where C: ChatClient {
    let client = C::default();

    loop {
        let msg = client
            .chat(options.clone(), &messages, chunk_sender.clone())
            .await?;

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

            //     if let Some(tool_calls) = tool_calls {
            //         // for tool_call in tool_calls {
            //         //     let tool_call_ret = tool_executor
            //         //         .exec(
            //         //             &tool_call.name,
            //         //             &tool_call.id,
            //         //             tool_call.arguments.as_deref(),
            //         //         )
            //         //         .await?;

            //         //     c.messages.push(Message::ToolMessage {
            //         //         content: tool_call_ret,
            //         //         tool_call_id: tool_call.id.clone(),
            //         //         name: tool_call.name.clone(),
            //         //     });
            //         // }
            //     }
        }
    }

    Ok(messages)
}
