use thiserror::Error;

use tokio::sync::mpsc;

#[derive(Error, Debug)]
pub enum TinyError {
    #[error("runtime error")]
    RuntimeError,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug)]
pub enum ContentPart {
    Text(String),
    Image { url: String, detail: ImageDetail },
    Video(String),
    File(String),
}

pub type RichContent = Vec<ContentPart>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum UserMessage {
    Text(String),
    Parts(RichContent),
}

#[derive(Debug)]
pub struct ToolCall {
    pub name: String,
    pub id: String,
    pub index: u32,
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

#[derive(Debug, Clone)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
    Other(String),
}

#[derive(Debug, Clone)]
pub enum ThinkingType {
    Enabled,
    Disabled,
    Adaptive,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct ThinkingOptions {
    pub t_type: ThinkingType,
    pub budget_tokens: u32,
}

impl Default for ThinkingOptions {
    fn default() -> Self {
        Self {
            t_type: ThinkingType::Enabled,
            budget_tokens: 8192,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatOptions {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub stream: bool,
    pub max_token: u32,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub thinking: Option<ThinkingOptions>,
    pub include_usage: bool,
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self {
            model: std::env::var("TINY_DEFAULT_MODEL").unwrap_or(Default::default()),
            base_url: std::env::var("TINY_DEFAULT_BASE_URL").unwrap_or(Default::default()),
            api_key: std::env::var("TINY_DEFAULT_API_KEY").unwrap_or(Default::default()),
            stream: true,
            include_usage: true,
            max_token: 8000,
            reasoning_effort: None,
            thinking: Some(Default::default()),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum MessageChunk {
    Chunk {
        content: Option<String>,
        reasoning_content: Option<String>,
        tool_calls: Option<Vec<ToolCall>>,
    },
    Error(String),
}

#[derive(Debug)]
pub struct ToolParameter {
    pub name: String,
    pub p_type: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

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

pub async fn tiny_loop<C, T>(
    options: &ChatOptions,
    mut messages: Vec<Message>,
    chat: C,
    tools: &Vec<Tool>,
    tool_execute: T,
    chunk_sender: mpsc::Sender<MessageChunk>,
) -> Result<Vec<Message>, TinyError>
where
    C: ChatAsyncFn,
    T: ToolAsyncFn,
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
                    let tool_call_ret = tool_execute(
                        &tool_call.name,
                        &tool_call.id,
                        tool_call.arguments.as_deref(),
                    )
                    .await?;

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
