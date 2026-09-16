use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc;

use crate::core::common::{ChatOptions, Message, MessageChunk, TinyError, tiny_loop};

// mod base;
mod core;
mod openai;
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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // let env = LuaEnv::new("tiny").unwrap();

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

    // env.exec_script("t.lua").await.unwrap();

    let options = ChatOptions {
        model: "qwen3.5".to_string(),
        base_url: "http://localhost:11434/v1".to_string(),
        api_key: "ollama".to_string(),
        max_token: 1024_000,
        ..Default::default()
    };

    let messages = vec![
        Message::SystemMessage("You are a helpful assistant.".to_string()),
        Message::UserMessage(core::common::UserMessage::Text("why is sky blue?".into())),
    ];
    let (tx, mut rx) = mpsc::channel::<MessageChunk>(1024);

    tokio::spawn(
        async move { tiny_loop(&options, messages, openai::chat, execute_tool, tx).await },
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
            }
            MessageChunk::Error(e) => {
                println!("{}", e)
            }
        }
    }
}
