use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub r#type: String,
    pub required: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Option<Vec<ToolParameter>>,
}
