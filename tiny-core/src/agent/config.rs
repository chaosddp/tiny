use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelFeatures {
    pub vision: bool,
    pub thinking: bool,
    pub decision: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelDefinition {
    pub model: String,
    pub provider: String,
    pub client: String,
    pub base_url: String,
    pub api_key: String,
    pub max_tokens: usize,
    pub features: ModelFeatures,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configurations {
    pub default_model: String,
    pub models: HashMap<String, ModelDefinition>,
    pub tools: Vec<String>,
    pub extensions: Vec<String>,
}
