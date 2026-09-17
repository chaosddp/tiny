mod core;
mod luaenv;
mod openai;

use std::io::Write;

use luaenv::{
    env::LuaEnv,
    lua::{Lua, LuaTable, LuaValue::Nil},
};
use mlua::Function;

use core::{
    ChatOptions, Message, MessageChunk, ReasoningEffort, ThinkingOptions, ThinkingType, TinyError,
    UserMessage, tiny_loop,
};
use tokio::sync::mpsc;

use crate::core::{Tool, ToolParameter};

// use luaenv::lua::{Lua, LuaExternalResult, LuaResult};

// use crate::lua_helpers::http::{LuaHttpClient, LuaHttpResponse};

// fn t(_: &Lua, _: ()) -> LuaResult<()> {
//     println!("hello, world");

//     Ok(())
// }

// fn create_new_http_client(_: &Lua, _: ()) -> LuaResult<LuaHttpClient> {
//     let client = LuaHttpClient::new().into_lua_err()?;

//     Ok(client)
// }

async fn execute_tool(name: &str, _id: &str, _args: Option<&str>) -> Result<String, TinyError> {
    if name == "get_weather" {
        Ok("Condition: Cloudy
        Temperature: 17°C (feels comfortable/cool)
        Humidity: 85%
        Wind: East at 4 mph
        High Temperature: 23°C
        Low Temperature: 15°C
        Tonight: Temperatures will drop to around 15°C and 14°C later tonight.
        "
        .to_string())
    } else {
        Ok("invalid".to_string())
    }
}

struct WChatOptions(ChatOptions);

impl From<&LuaTable> for WChatOptions {
    fn from(value: &LuaTable) -> Self {
        let mut options = ChatOptions::default();

        if let Ok(chat_table) = value.get::<LuaTable>("chat") {
            options.model = chat_table.get::<String>("model").unwrap_or_default();
            options.base_url = chat_table.get::<String>("base_url").unwrap_or_default();
            options.api_key = chat_table.get::<String>("api_key").unwrap_or_default();
            options.max_token = chat_table.get::<u32>("max_token").unwrap_or(8000);
            options.reasoning_effort = Some(match chat_table.get::<String>("reasoning_effort") {
                Ok(effort) => match effort.as_ref() {
                    "low" => ReasoningEffort::Low,
                    "medium" => ReasoningEffort::Medium,
                    "high" => ReasoningEffort::High,
                    _ => ReasoningEffort::Other(effort),
                },
                _ => ReasoningEffort::Medium,
            });

            if let Ok(thinking_table) = chat_table.get::<LuaTable>("thinking") {
                let mut thinking = ThinkingOptions::default();

                if let Ok(ttype) = thinking_table.get::<String>("type") {
                    thinking.t_type = match ttype.as_ref() {
                        "enabled" => ThinkingType::Enabled,
                        "disabled" => ThinkingType::Disabled,
                        "adaptive" => ThinkingType::Adaptive,
                        _ => ThinkingType::Other(ttype),
                    };
                }

                if let Ok(budget_tokens) = thinking_table.get::<u32>("budget_tokens") {
                    thinking.budget_tokens = budget_tokens;
                }
            }
        }

        WChatOptions(options)
    }
}

struct WLuaTable(LuaTable);

impl From<(&Lua, ChatOptions)> for WLuaTable {
    fn from(value: (&Lua, ChatOptions)) -> Self {
        let lua = value.0;
        let options = value.1;

        let config_table = lua.create_table().unwrap();

        // options of chat
        let chat_options_table = lua.create_table().unwrap();

        // TODO: not supported yet
        chat_options_table.set("provider", Nil).unwrap();
        chat_options_table
            .set("model", options.model.to_string())
            .unwrap();
        chat_options_table
            .set("base_url", options.base_url.to_string())
            .unwrap();
        chat_options_table
            .set("api_key", options.api_key.to_string())
            .unwrap();
        chat_options_table
            .set("max_tokens", options.max_token)
            .unwrap();
        chat_options_table
            .set(
                "reasoning_effort",
                match &options.reasoning_effort {
                    Some(effort) => match effort {
                        ReasoningEffort::Low => "low",
                        ReasoningEffort::Medium => "medium",
                        ReasoningEffort::High => "high",
                        ReasoningEffort::Other(s) => &s,
                    },
                    _ => "low",
                },
            )
            .unwrap();

        let thinking_options_table = lua.create_table().unwrap();

        if let Some(thinking) = &options.thinking {
            thinking_options_table
                .set(
                    "type",
                    match &thinking.t_type {
                        ThinkingType::Enabled => "enabled",
                        ThinkingType::Disabled => "disabled",
                        ThinkingType::Adaptive => "adaptive",
                        ThinkingType::Other(s) => &s,
                    },
                )
                .unwrap();

            thinking_options_table
                .set("budget_tokens", thinking.budget_tokens)
                .unwrap();
        } else {
            thinking_options_table.set("type", "enabled").unwrap();
            thinking_options_table.set("budget_tokens", 8192).unwrap();
        }

        chat_options_table
            .set("thinking", thinking_options_table)
            .unwrap();

        let tools_table = lua.create_table().unwrap();

        config_table.set("tools", tools_table).unwrap();
        config_table.set("chat", chat_options_table).unwrap();

        WLuaTable(config_table)
    }
}

struct Tools(Vec<Tool>);

impl From<&LuaTable> for Tools {
    fn from(value: &LuaTable) -> Self {
        let mut tools = vec![];

        for pair in value.pairs::<String, LuaTable>() {
            if let Ok((name, tool_definition)) = pair {
                let description = tool_definition
                    .get::<String>("desc")
                    .unwrap_or(name.clone());

                if let Ok(func) = tool_definition.get::<Function>("func") {
                    // here it is a valid tool definition
                    let mut tool = Tool {
                        name: name,
                        description: description,
                        parameters: vec![],
                    };

                    if let Ok(parameters) = tool_definition.get::<LuaTable>("parameters") {
                        for p_pair in parameters.pairs::<String, LuaTable>() {
                            if let Ok((p_name, p_definition)) = p_pair {
                                let p_type = p_definition
                                    .get::<String>("type")
                                    .unwrap_or("string".to_string());
                                let p_desc =
                                    p_definition.get::<String>("desc").unwrap_or(p_name.clone());
                                let p_required =
                                    p_definition.get::<bool>("required").unwrap_or_default();

                                tool.parameters.push(ToolParameter {
                                    name: p_name,
                                    p_type: p_type,
                                    description: p_desc,
                                    required: p_required,
                                });
                            }
                        }
                    }

                    tools.push(tool);
                }
            }
        }

        Tools(tools)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    let env = LuaEnv::new("tiny").unwrap();

    // load the entry script
    env.exec_script(".tiny/main.lua").await.unwrap();

    let config_table = WLuaTable::from((&env.weak().upgrade(), ChatOptions::default())).0;

    // call the config function
    env.call::<()>("tiny.conf", &config_table).await.unwrap();

    let options = WChatOptions::from(&config_table).0;

    let messages = vec![
        Message::SystemMessage("You are a helpful assistant.".to_string()),
        Message::UserMessage(UserMessage::Text("what is the weather in Beijing?".into())),
    ];

    let tools = Tools::from(
        &config_table
            .get::<LuaTable>("tools")
            .unwrap_or(env.weak().upgrade().create_table().unwrap()),
    )
    .0;

    let (tx, mut rx) = mpsc::channel::<MessageChunk>(1024);

    tokio::spawn(async move {
        tiny_loop(&options, messages, openai::chat, &tools, execute_tool, tx).await
    });

    while let Some(msg) = rx.recv().await {
        match msg {
            MessageChunk::Chunk {
                content,
                reasoning_content,
                tool_calls,
            } => {
                if let Some(c) = content {
                    print!("{}", c);
                    std::io::stdout().flush().unwrap();
                }

                if let Some(rc) = reasoning_content {
                    print!("{}", rc);
                    std::io::stdout().flush().unwrap();
                }

                if let Some(tc_list) = tool_calls {
                    println!("\n{:?}", tc_list);
                }
            }
            MessageChunk::Error(e) => {
                println!("{}", e)
            }
        }
    }
}
