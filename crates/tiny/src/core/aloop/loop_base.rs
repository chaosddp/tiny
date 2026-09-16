// use std::{
//     cell::RefCell,
//     sync::{Arc, Mutex},
// };

// use async_trait::async_trait;
// use thiserror::Error;

// #[derive(Error, Debug)]
// pub enum LoopError {
//     #[error("runtime error")]
//     RuntimeError,
// }

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub enum ContentPart {
//     Text(String),
//     Image { url: String, detail: String },
//     Video(String),
//     File(String),
// }

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub enum UserMessage {
//     Text(String),
//     ContentParts(Vec<ContentPart>),
// }

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct AssistantMessage {
//     pub content: Option<String>,
//     pub reasoning_content: Option<String>,
//     pub reasoning_details: Option<Vec<String>>,
//     pub tool_calls: Option<Vec<ToolCall>>,
//     pub finished_reason: FinishReason,
// }

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub enum FinishReason {
//     ToolCall,
//     Other,
// }

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct ToolCall {
//     pub name: String,
//     pub id: String,
//     pub arguments: Option<String>,
// }

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub enum Message {
//     SystemMessage(String),
//     UserMessage(UserMessage),
//     AssistantMessage(AssistantMessage),
//     ToolMessage {
//         content: String,
//         tool_call_id: String,
//         name: String,
//     },
// }

// pub enum MessageChunk {
//     Chunk {
//         content: Option<String>,
//         reasoning_content: Option<String>,
//         tool_calls: Option<Vec<ToolCall>>,
//     },
//     Error(String),
// }

// pub struct Context {
//     pub messages: Vec<Message>,
//     pub tool_executor: Box<dyn ToolExecutor>,
// }

// pub struct Configurations {
//     pub model: String,
//     pub base_url: String,
//     pub api_key: String,
//     pub max_tokens: u32,
// }

// #[async_trait]
// pub trait ToolExecutor {
//     async fn exec(
//         &self,
//         name: &str,
//         id: &str,
//         arguments: Option<&str>,
//     ) -> Result<String, LoopError>;
// }

// // pub async fn base_loop<SF, SFt, CF, CFt>(
// //     ctx: Arc<Mutex<Context>>,
// //     config: &Configurations,
// //     stream_chat: AsyncFn(&str, &str, &str, u32, &Vec<Message>, AsyncFn(MessageChunk)),
// //     chunk_reciever: CF,
// // ) -> Result<(), LoopError>
// // where
// //     SF: Fn(&str, &str, &str, u32, &Vec<Message>, &CF) -> SFt,
// //     SFt: Future<Output = Result<Message, LoopError>>,
// //     CF: Fn(MessageChunk) -> CFt,
// //     CFt: Future<Output = ()>,
// // {
// //     loop {
// //         let mut c = ctx.lock().unwrap();

// //         let msg = stream_chat(
// //             &config.model,
// //             &config.base_url,
// //             &config.api_key,
// //             config.max_tokens,
// //             &c.messages,
// //             &chunk_reciever,
// //         )
// //         .await?;

// //         if let Message::AssistantMessage(amsg) = &msg {
// //             if amsg.finished_reason == FinishReason::ToolCall || amsg.tool_calls.is_none() {
// //                 c.messages.push(msg);

// //                 break;
// //             }

// //             if let Some(tool_calls) = &amsg.tool_calls {
// //                 for tool_call in tool_calls {
// //                     let tool_call_ret = c
// //                         .tool_executor
// //                         .exec(
// //                             &tool_call.name,
// //                             &tool_call.id,
// //                             tool_call.arguments.as_deref(),
// //                         )
// //                         .await?;

// //                     c.messages.push(Message::ToolMessage {
// //                         content: tool_call_ret,
// //                         tool_call_id: tool_call.id.clone(),
// //                         name: tool_call.name.clone(),
// //                     });
// //                 }
// //             }
// //         }
// //     }

// //     Ok(())
// // }

