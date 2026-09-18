mod agent;
mod core;
mod luaenv;
mod openai;

use std::{collections::HashMap, io::Write};

use log::debug;
use luaenv::{env::LuaEnv, lua::*};

use core::{ChatOptions, Message, MessageChunk, UserMessage, tiny_loop};
use serde_json::Value as JsonValue;
use tokio::sync::mpsc;

use crate::{
    agent::types::{Tools, WChatOptions, WLuaTable},
    core::Tool,
};

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

    let lua_tools: Vec<agent::types::LuaFuncTool> = Tools::from(
        &config_table
            .get::<LuaTable>("tools")
            .unwrap_or(env.weak().upgrade().create_table().unwrap()),
    )
    .0;

    let mut tools: Vec<Tool> = Vec::new();
    let mut lua_tool_functions: HashMap<String, LuaFunction> = HashMap::new();

    for tool_pair in lua_tools {
        let tool_name = tool_pair.0.0.name.to_string();

        debug!("lua tool function: {} -> {:?}", tool_name, tool_pair.0.1);

        tools.push(tool_pair.0.0);
        lua_tool_functions.insert(tool_name, tool_pair.0.1);
    }

    let (chunk_tx, mut chunk_rx) = mpsc::channel::<MessageChunk>(1024);
    let (tool_call_tx, mut tool_call_rx) = mpsc::channel::<(String, String, Option<String>)>(1);
    let (tool_complete_tx, mut tool_complet_rx) = mpsc::channel::<String>(1);

    tokio::spawn(async move {
        tiny_loop(
            &options,
            messages,
            openai::chat,
            &tools,
            (tool_call_tx, tool_complet_rx),
            chunk_tx,
        )
        .await
    });

    tokio::spawn(async move {
        while let Some(msg) = chunk_rx.recv().await {
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
    });

    while let Some(tool_call) = tool_call_rx.recv().await {
        debug!(
            "recieve tool call ({}): {}({:?})",
            tool_call.1, tool_call.0, tool_call.2
        );

        if tool_call.0.len() > 0 && tool_call.1.len() > 0 {
            if let Some(func) = lua_tool_functions.get(&tool_call.0) {
                debug!("found function: {:?}", func.info());

                if let Some(args) = tool_call.2 {
                    // parse it to json object, then to lua table
                    match serde_json::from_str::<JsonValue>(&args) {
                        Ok(args_json_value) if args_json_value.is_object() => {
                            // we use the first level as parameters
                            // let mut args_map: HashMap<String, LuaValue> = HashMap::new();
                            let args_table = env.weak().upgrade().create_table().unwrap();

                            for (k, v) in args_json_value.as_object().unwrap() {
                                // TODO: support nested parameter type
                                match v {
                                    JsonValue::Bool(b) => args_table.set(k.clone(), *b).unwrap(),
                                    JsonValue::Null => {
                                        args_table.set(k.clone(), LuaValue::Nil).unwrap()
                                    }
                                    JsonValue::Number(n) => {
                                        args_table.set(k.clone(), n.as_f64()).unwrap()
                                    }
                                    JsonValue::String(s) => {
                                        args_table.set(k.clone(), s.clone()).unwrap()
                                    }
                                    _ => {}
                                }
                            }

                            let ret = func.call::<String>(args_table).unwrap();
                            tool_complete_tx.send(ret).await.unwrap();
                        }
                        _ => {
                            tool_complete_tx
                                .send(
                                    "Invalid tool call parameters, need a valid json object."
                                        .to_string(),
                                )
                                .await
                                .unwrap();
                        }
                    }
                } else {
                    let ret = func.call::<String>(()).unwrap();
                    tool_complete_tx.send(ret).await.unwrap();
                }
            }
        } else {
            debug!(
                "avaiable tools: {:?}",
                lua_tool_functions
                    .keys()
                    .map(|k| k.clone())
                    .collect::<String>()
            );

            tool_complete_tx
                .send("Invalid tool call".to_string())
                .await
                .unwrap();
        }
    }
}
