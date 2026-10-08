use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Choice {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Quest {
    Noul(String),
    Choice {
        instructions: String,
        criteria: Vec<Choice>,
    },
    Score {
        instructions: String,
        criteria: Vec<String>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum State {
    Text(String),
    Array(Vec<String>),
    Object(HashMap<String, String>),
}
