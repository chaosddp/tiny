/// OpenAI compatible "Image resolution policy" with customize support, Default is "auto".
///
/// Use [`ImageDetail::Other`] for your LLM providers.
///
/// # Example
///
/// ```
/// # use tiny_core::ImageDetail;
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

/// user message content part that support text, image, video and file.
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

/// user message, support plain text and rich content with parts.
///
/// # Example
///
/// ```
/// # use tiny_core::types::{UserMessage, ContentPart};
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

/// tool call message from model.
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

#[derive(Debug)]
pub struct AssistantMessage {
    pub content: Option<String>,
    pub reasoning_content: Option<String>,
    pub reasoning_details: Option<Vec<String>>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub finished_reason: FinishReason,
}

#[derive(Debug)]
pub struct ToolMessage {
    pub content: String,
    pub tool_call_id: String,
    pub name: String,
}

/// Message represents a message from/to the LLM, role will be assigned according to it variant.
///
/// # Example
///
/// ```
/// # use tiny_core::types::{Message, UserMessage, FinishReason};
///
/// // usually for prompt
/// let system_message = Message::System("You are a helpful assistant.".to_string());
///
/// let user_message = Message::User(UserMessage::Text("hello".to_string()));
///
/// // usually assistant message is from LLM.
/// let assistant_message = Message::Assistant(AssistantMessage{
///     content: Some("hello from LLM...".to_string()),
///     reasoning_content: None,
///     reasoning_details: None,
///     tool_calls: None,
///     finished_reason: FinishReason::Stop
/// });
///
/// // usually tool message is from loop, wrap the result of tool result
/// let tool_message = Message::Tool {
///     content: "result from a tool".to_string(),
///     name: "name_of_tool".to_string(),
///     tool_call_id: "tool_call_id_from_LLM".to_string()
/// };
/// ```
#[derive(Debug)]
pub enum Message {
    System(String),
    User(UserMessage),
    Assistant(AssistantMessage),
    Tool(ToolMessage),
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
