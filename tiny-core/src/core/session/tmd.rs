use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Write,
};

use crate::core::TinyResult;

/// ### level 3 header
#[derive(Debug)]
pub struct Item {
    pub title: String,
    pub content: String,
}

pub struct Writer {
    file: File,
}

impl Writer {
    pub fn new(file: &str) -> TinyResult<Self> {
        Ok(Writer {
            file: OpenOptions::new().append(true).open(file)?,
        })
    }
    
    // TODO: use &str instead String
    pub fn add(
        &mut self,
        title: &str,
        meta: Option<HashMap<String, String>>,
        content: Option<String>,
        items: Option<Vec<Item>>,
    ) -> TinyResult<()> {
        self.file.write(&format!("## {}\n", title).as_bytes())?;

        if let Some(meta_data) = meta {
            let meta_str = meta_data
                .iter()
                .map(|e| format!("{}: {}", e.0, e.1))
                .collect::<Vec<String>>()
                .join("\n");

            self.file.write(&meta_str.as_bytes())?;
        }

        if let Some(content) = content {
            self.file.write(&format!("\n{}", content).as_bytes())?;
        }

        if let Some(items) = items {
            let items_str = items
                .iter()
                .map(|e| format!("## {}\n\n{}", e.title, e.content))
                .collect::<Vec<String>>()
                .join("\n\n");

            self.file.write(&format!("\n{}", items_str).as_bytes())?;
        }

        self.file.flush()?;

        Ok(())
    }
}
