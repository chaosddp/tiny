use std::{fs::File, io::Read, path::Path};

use base64::{Engine, engine::general_purpose};
use thiserror::Error;

/// Error from this application
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum TinyError {
    #[error("Input file not exist: {0}")]
    InputFileNotExist(String),
    #[error("Error when open file: {0}")]
    IoError(std::io::Error),
    #[error("Invalid image file: {0}")]
    InvalidImageFile(String),
    #[error("Fail to call lua content: {0}")]
    InvalidLuaContent(mlua::Error),
    #[error("Invalid lua trait object: {0}")]
    InvalidLuaTraitObject(String),
    #[error("Lua reference droped.")]
    InvalidLuaReference,
    #[error("Invalid json object: {0}")]
    InvalidJsonObject(String),
    #[error("runtime error")]
    RuntimeError,
}

impl From<std::io::Error> for TinyError {
    fn from(value: std::io::Error) -> Self {
        TinyError::IoError(value)
    }
}

impl From<mlua::Error> for TinyError {
    fn from(value: mlua::Error) -> Self {
        TinyError::InvalidLuaContent(value)
    }
}

/// OpenAI compatible "Image resolution policy" with customize support, Default is "auto".
///
/// Use [`ImageDetail::Other`] for your LLM providers.
///
/// # Example
///
/// ```
/// # use tiny::core::ImageDetail;
///
/// // use default policy
/// let image_detail = ImageDetail::default();
///
/// assert_eq!(ImageDetail::Auto, image_detail);
///
/// // use costomized policy
/// let my_detail = ImageDetail::Other("my_policy");
///
/// ```
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
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

/// OpenAI compatible user message content part that support text, image, video and file.
///
/// Used to construct rich user message. See [`UserMessage`] for usage.
#[allow(dead_code)]
#[derive(Debug)]
pub enum ContentPart {
    Text(String),
    Image { url: String, detail: ImageDetail },
    Video(String),
    File(String),
}

/// OpenAI compatible user message, support plain text and rich content with parts.
///
/// # Example
///
/// ```
/// # use tiny::core::types::{UserMessage, ContentPart};
///
/// let text_message = UserMessage::Text("hello.".to_string());
///
/// let complex_message = UserMessage::Parts(vec![
///     ContentPart::Text("What is in the image?".to_string()),
///     ContentPart::Image{url: "base64 image data".to_string(), detail: Default::default()}
///
/// ]);
///
/// ```
#[allow(dead_code)]
#[derive(Debug)]
pub enum UserMessage {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// OpenAi compatible tool call message from model.
#[allow(dead_code)]
#[derive(Debug)]
pub struct ToolCall {
    pub name: String,
    pub id: String,
    pub index: u32,
    pub arguments: Option<String>,
}

/// The reason for generation completion
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum FinishReason {
    /// Normal termination
    Stop,
    /// The max_tokens limit is reached, and the output is truncated
    Length,
    /// The model needs to call a tool
    ToolCall,
    /// Content is filtered by the security policy
    ContentFilter,
    /// Any other customize reason
    Other(String),
}

/// Message represents a message from/to the LLM, role will be assigned according to it variant.
///
/// # Example
///
/// ```
/// # use tiny::core::types::{Message, UserMessage, FinishReason};
///
/// // usually for prompt
/// let system_message = Message::System("You are a helpful assistant.".to_string());
///
/// let user_message = Message::User(UserMessage::Text("hello".to_string()));
///
/// // usually assistant message is from LLM.
/// let assistant_message = Message::Assistant{
///     content: Some("hello from LLM...".to_string()),
///     reasoning_content: None,
///     reasoning_details: None,
///     tool_calls: None,
///     finished_reason: FinishReason::Stop
/// };
///
/// // usually tool message is from loop, wrap the result of tool result
/// let tool_message = Message::Tool {
///     content: "result from a tool".to_string(),
///     name: "name_of_tool".to_string(),
///     tool_call_id: "tool_call_id_from_LLM".to_string()
/// };
///
/// // or use short-cut:
///
/// let system_message = Message::system("You are a helpful assistant.");
///
/// let user_message = Message::user("hello");
///
/// // this function will provide default values to others fields
/// let assistant_message = Message::assistant("hello from LLM...");
///
/// let tool_message = Message::tool("my_tool", "my_call_id", "my_tool_result");
///
/// if let Ok(user_image_message) = Message::image("What is in the image?", "/path/to/file/") {
///     // other code
/// }
/// ```
#[derive(Debug)]
pub enum Message {
    System(String),
    User(UserMessage),
    Assistant {
        content: Option<String>,
        reasoning_content: Option<String>,
        reasoning_details: Option<Vec<String>>,
        tool_calls: Option<Vec<ToolCall>>,
        finished_reason: FinishReason,
    },
    Tool {
        content: String,
        tool_call_id: String,
        name: String,
    },
}

#[allow(dead_code)]
impl Message {
    /// Create a system message with prompt
    pub fn system(prompt: &str) -> Message {
        Message::System(prompt.to_string())
    }

    /// Create a text user message
    pub fn user(message: &str) -> Message {
        Message::User(UserMessage::Text(message.to_string()))
    }

    /// Create a user message with text and image
    pub fn image(message: &str, image_file: &str) -> Result<Message, TinyError> {
        let image_path = Path::new(image_file);

        if !image_path.exists() {
            return Err(TinyError::InputFileNotExist(image_file.to_string()));
        }

        let mut buffer = Vec::new();
        {
            let mut fp = File::open(image_file)?;
            let _out = fp.read_to_end(&mut buffer)?;
        }

        let data = general_purpose::STANDARD.encode(buffer);
        // TODO: we can use magic number here to do extension checking
        let ext = image_path.extension().unwrap_or_default();

        if ext.len() == 0 {
            return Err(TinyError::InvalidImageFile(image_file.to_string()));
        }

        let image_data_url = format!("data:image/{};base64,{}", ext.to_str().unwrap(), data);

        Ok(Message::User(UserMessage::Parts(vec![
            ContentPart::Text(message.to_string()),
            ContentPart::Image {
                url: image_data_url,
                detail: Default::default(),
            },
        ])))
    }

    /// Create an asssistant message with text content only
    pub fn assistant(content: &str) -> Message {
        Message::Assistant {
            content: Some(content.to_string()),
            reasoning_content: None,
            reasoning_details: None,
            tool_calls: None,
            finished_reason: FinishReason::Stop,
        }
    }

    pub fn tool(name: &str, id: &str, content: &str) -> Message {
        Message::Tool {
            content: content.to_string(),
            tool_call_id: id.to_string(),
            name: name.to_string(),
        }
    }
}

/// Reasoning depth. It is applicable to reasoning models, User [`ReasoningEffort::Other`] for customize depth.
#[derive(Debug, Clone)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
    Other(String),
}

/// Thinking type
#[derive(Debug, Clone)]
pub enum ThinkingType {
    /// for enabling
    Enabled,
    /// for disabling
    Disabled,
    /// for adaptive mode
    Adaptive,
    Other(String),
}

/// Thinking mode settings
#[derive(Debug, Clone)]
pub struct ThinkingOptions {
    pub t_type: ThinkingType,
    /// The maximum number of tokens for the reasoning process, default is 8192
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

/// Options for chat interface
///
/// The [`Default`] implementation will try to read model, base_url and api_key from environment variables with following name:
/// - TINY_DEFAULT_MODEL: model name
/// - TINY_DEFAULT_BASE_URL: api base url
/// - TINY_DEFAULT_API_KEY: api key
#[derive(Debug, Clone)]
pub struct ChatOptions {
    /// Model identifier: like 'deepseek-v4-flash', 'qwen3.5'
    pub model: String,
    /// Base of the provider api interface
    pub base_url: String,
    /// Api key to access the api
    pub api_key: String,
    /// Whether to enable streaming output (SSE)
    pub stream: bool,
    /// The maximum number of tokens that can be generated in a single response
    pub max_token: u32,
    /// reasoning depth
    pub reasoning_effort: Option<ReasoningEffort>,
    /// thinking options
    pub thinking: Option<ThinkingOptions>,
    /// Whether the last chunk in streaming mode contains token usage, for stream=true only.
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

/// Chunk from server side when stream mode enabled
#[allow(dead_code)]
#[derive(Debug)]
pub enum MessageChunk {
    Chunk {
        content: Option<String>,
        reasoning_content: Option<String>,
        tool_calls: Option<Vec<ToolCall>>,
        tool_result: Option<String>,
    },
    Error(String),
}

/// Tool parameter definition
#[derive(Debug)]
pub struct ToolParameter {
    /// name of the parameter, it should be a valid variable name
    pub name: String,
    /// type of the parameter
    pub p_type: String,
    /// description of the parameter
    pub description: String,
    /// if the parameter is required
    pub required: bool,
}

/// Tool definition
#[derive(Debug)]
pub struct Tool {
    /// name of the tool
    pub name: String,
    /// description of the tool
    pub description: String,
    /// parameters of the tool
    pub parameters: Vec<ToolParameter>,
}
