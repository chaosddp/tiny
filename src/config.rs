use mlua::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelFeatures {
    pub vision: bool,
    pub thinking: bool,
    pub tools: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelDefinition {
    pub name: String,                  // unique name of the model definition
    pub client: String,                // client type of the model, openai or any other
    pub base_url: String,              // base url
    pub api_key: String,               // api key, can be a plain text or from environment variable
    pub max_tokens: usize,             // max token to use
    pub thinking_efforts: Vec<String>, // thinking efforts the model support
    pub features: ModelFeatures,       // what features we support
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configurations {
    pub default_model: Option<String>,
    pub models: Option<Vec<ModelDefinition>>,
    pub tools: Option<Vec<String>>,
    pub extensions: Option<Vec<String>>,
}

impl IntoLua for ModelFeatures {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let t = lua.create_table()?;

        t.set("vision", self.vision)?;
        t.set("thinking", self.thinking)?;
        t.set("tools", self.tools)?;

        Ok(LuaValue::Table(t))
    }
}

impl IntoLua for &ModelDefinition {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let t = lua.create_table()?;

        let name_parts: Vec<&str> = self.name.split("/").collect();

        if name_parts.len() != 2 {
            return Err(LuaError::RuntimeError(format!(
                "invalid model name: {}, model name should be like 'provider/model_name'",
                self.name
            )));
        }

        let provider = name_parts[0];
        let model_name = name_parts[1];

        t.set("provider", provider.to_string())?;
        t.set("model", model_name.to_string())?;
        t.set("base_url", model_name.to_string())?;

        if self.api_key.starts_with("env:") {
            let env_key = &self.api_key[4..];

            let api_key_from_env = std::env::var(env_key).unwrap_or_default();

            t.set("api_key", api_key_from_env)?;
        } else {
            t.set("api_key", self.api_key.to_string())?;
        }

        t.set("max_tokens", self.max_tokens)?;
        t.set("features", self.features.clone())?;

        Ok(LuaValue::Table(t))
    }
}

impl mlua::IntoLua for &Configurations {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let t = lua.create_table()?;

        if let Some(model) = &self.default_model {
            t.set("default_model", model.to_string())?;
        }

        if let Some(models) = &self.models {
            let models_table = lua.create_table_with_capacity(0, models.len())?;

            for model in models {
                models_table.set(model.name.to_string(), model)?;
            }

            t.set("models", models_table)?;
        }

        if let Some(tools) = &self.tools {
            t.set("tools", tools.clone())?;
        }

        if let Some(extensions) = &self.extensions {
            t.set("extensions", extensions.clone())?;
        }

        Ok(LuaValue::Table(t))
    }
}

impl Default for Configurations {
    fn default() -> Self {
        let name = "deepseek/deepseek-flash";

        Self {
            default_model: Some(name.into()),
            models: Some(vec![ModelDefinition {
                name: name.into(),
                client: "openai".into(),
                base_url: "https://api.deepseek.com".into(),
                api_key: "env:TINY_DEEPSEEK_APIKEY".into(),
                max_tokens: 1024_000,
                thinking_efforts: vec!["low".into(), "medium".into(), "high".into()],
                features: ModelFeatures {
                    vision: true,
                    thinking: true,
                    tools: true,
                },
            }]),
            tools: Some(vec!["extensions/default".into()]),
            extensions: Some(vec!["tools/default".into()]),
        }
    }
}
