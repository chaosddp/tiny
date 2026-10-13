use std::collections::HashMap;

use crate::core::{
    TinyResult,
    chat::messages::{ChatMessage, FinishReason, UserContentPart, UserMessage},
    session::tmd::{Item, Writer},
};

pub mod tmd;

pub struct MemoryWriter(Writer);

impl MemoryWriter {
    pub fn add(&mut self, message: &ChatMessage) -> TinyResult<()> {
        let mut meta =
            HashMap::from_iter([("datetime".to_string(), "xxxx-xx-xx xx:xx:xx".to_string())]);

        match message {
            ChatMessage::System(prompt) => {
                self.0
                    .add("system", Some(meta), Some(prompt.to_string()), None)?;
            }
            ChatMessage::User(msg) => match msg {
                UserMessage::Text(content) => {
                    self.0
                        .add("user", Some(meta), Some(content.to_string()), None)?;
                }
                UserMessage::Parts(parts) => {
                    let mut items = vec![];

                    for part in parts {
                        match part {
                            UserContentPart::Text(text) => {
                                items.push(Item {
                                    title: "text part".into(),
                                    content: text.to_string(),
                                });
                            }
                            UserContentPart::File(file) => {
                                items.push(Item {
                                    title: "file part".into(),
                                    content: file.to_string(),
                                });
                            }
                            UserContentPart::Video(video) => {
                                items.push(Item {
                                    title: "video part".into(),
                                    content: video.to_string(),
                                });
                            }
                            UserContentPart::Image { image, detail: _ } => {
                                items.push(Item {
                                    title: "image part".into(),
                                    content: image.to_string(),
                                });
                            }
                        }
                    }
                }
            },
            ChatMessage::Assistant(msg) => {
                meta.insert(
                    "finish_reason".into(),
                    match &msg.finish_reason {
                        FinishReason::Stop => "stop".into(),
                        FinishReason::Length => "length".into(),
                        FinishReason::ToolCall => "tool_call".into(),
                        FinishReason::ContentFilter => "content_filter".into(),
                        FinishReason::Other(s) => s.to_string(),
                    },
                );

                if let Some(usage) = &msg.usage {
                    meta.insert("prompt_tokens".into(), usage.prompt.to_string());
                    meta.insert("completion_tokens".into(), usage.completion.to_string());
                }

                let mut items = Vec::with_capacity(3);

                if let Some(reasoning) = &msg.reasoning {
                    items.push(Item {
                        title: "reasoning".into(),
                        content: reasoning.to_string(),
                    });
                }

                if let Some(content) = &msg.content {
                    items.push(Item {
                        title: "content".into(),
                        content: content.to_string(),
                    })
                }

                if let Some(tool_calls) = &msg.tool_calls {
                    items.push(Item {
                        title: "tool calls".into(),
                        content: serde_json::to_string_pretty(tool_calls)?,
                    });
                }
            }
            ChatMessage::Tool(msg) => {
                meta.insert("id".into(), msg.id.to_string());
                meta.insert("name".into(), msg.name.to_string());

                self.0
                    .add("tool", Some(meta), Some(msg.content.to_string()), None)?;
            }
        }

        Ok(())
    }
}

pub struct Session {
    
}

impl Session {
    pub fn new(folder: &str)->Self {
        
    }

    pub fn add_message(&mut self, message: &ChatMessage) -> TinyResult<()> {
        todo!()
    }

    pub fn read_memory(&self) -> TinyResult<Vec<ChatMessage>> {
        todo!()
    }

    pub fn add_trace(&mut self) -> TinyResult<()> {
        todo!()
    }
}
