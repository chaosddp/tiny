use std::io::Write;

use luaenv::{
    env::LuaEnv,
    lua::{LuaTable, LuaValue::Nil},
};

use llm::core::{
    ChatOptions, Message, MessageChunk, ReasoningEffort, ThinkingOptions, ThinkingType, TinyError,
    UserMessage, tiny_loop,
};
use tokio::sync::mpsc;

// mod ollama;
mod lua_helpers;

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

async fn execute_tool(name: &str, id: &str, args: Option<&str>) -> Result<String, TinyError> {
    if name == "weather" {
        Ok("30 ℃".to_string())
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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let env = LuaEnv::new("tiny").unwrap();

    //     env.add_function("test", t).unwrap();
    //     env.add_function("a.b.test", t).unwrap();
    //     env.add_member("a.b.pi", 3.1415).unwrap();
    //     env.add_package_path("r/?.lua").unwrap();

    //     env.exec_script("t.lua").await.unwrap();

    //     env.call::<()>("print", ("hello", "-", "world"))
    //         .await
    //         .unwrap();

    //     let ret = env.call::<i32>("sum", (1, 2)).await.unwrap();

    //     println!("a + b = {}", ret);

    // env.add_function("http.newClient", create_new_http_client)
    //     .unwrap();

    let config_table = env.weak().upgrade().create_table().unwrap();

    // options of chat
    let chat_options_table = env.weak().upgrade().create_table().unwrap();

    chat_options_table.set("provider", Nil).unwrap();
    chat_options_table.set("model", Nil).unwrap();
    chat_options_table.set("base_url", Nil).unwrap();
    chat_options_table.set("api_key", Nil).unwrap();
    chat_options_table.set("max_tokens", 8000).unwrap();
    chat_options_table.set("reasoning_effort", "low").unwrap();

    let thinking_options_table = env.weak().upgrade().create_table().unwrap();

    thinking_options_table.set("type", "enabled").unwrap();
    thinking_options_table.set("budget_tokens", 8192).unwrap();

    chat_options_table
        .set("thinking", thinking_options_table)
        .unwrap();

    let tools_table = env.weak().upgrade().create_table().unwrap();

    config_table.set("tools", tools_table).unwrap();
    config_table.set("chat", chat_options_table).unwrap();

    // load the entry script
    env.exec_script(".tiny/main.lua").await.unwrap();

    // call the config function
    env.call::<()>("tiny.conf", &config_table).await.unwrap();

    let options = WChatOptions::from(&config_table).0;

    let messages = vec![
        Message::SystemMessage("You are a helpful assistant.".to_string()),
        Message::UserMessage(UserMessage::Text("why is sky blue?".into())),
    ];
    let (tx, mut rx) = mpsc::channel::<MessageChunk>(1024);

    tokio::spawn(
        async move { tiny_loop(&options, messages, llm_openai::chat, execute_tool, tx).await },
    );

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
                } else if let Some(rc) = reasoning_content {
                    print!("{}", rc);
                    std::io::stdout().flush().unwrap();
                }
                // println!("{:?}", content);
            }
            MessageChunk::Error(e) => {
                println!("{}", e)
            }
        }
    }
}
