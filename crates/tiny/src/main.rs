mod base;
mod core;
mod lua_helpers;

use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use luaenv::env::LuaEnv;
use luaenv::lua::{Lua, LuaExternalResult, LuaResult};

use crate::base::chat_provider::openai_compatible_stream;
use crate::base::tool_executor::DefaultToolExecutor;
use crate::core::aloop::loop_base::{LoopError, Message, MessageChunk, UserMessage};
use crate::lua_helpers::http::{LuaHttpClient, LuaHttpResponse};

use core::aloop::loop_base::{Configurations, Context, base_loop};

fn t(_: &Lua, _: ()) -> LuaResult<()> {
    println!("hello, world");

    Ok(())
}

fn create_new_http_client(_: &Lua, _: ()) -> LuaResult<LuaHttpClient> {
    let client = LuaHttpClient::new().into_lua_err()?;

    Ok(client)
}

async fn on_chunk(chunk: MessageChunk) {
    match chunk {
        MessageChunk::Chunk {
            content,
            reasoning_content,
            tool_calls,
        } => {
            if let Some(c) = content && c.len() > 0 {
                print!("{}", c);
            }
        }
        MessageChunk::Error(e) => {
            println!("error: {}", e);
        }
    }
}

async fn a<F, Ft>(
    model: &str,
    base_url: &str,
    api_key: &str,
    max_tokens: u32,
    messages: &Vec<Message>,
    chunk_reciever: &F,
) -> Result<Message, LoopError>
where
    F: Fn(MessageChunk) -> Ft,
    Ft: Future<Output = ()>,
{
    Err(LoopError::RuntimeError)
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

    let conf = Configurations {
        model: "qwen3.5".to_string(),
        base_url: "http://localhost:11434/v1".to_string(),
        api_key: "Ollama".to_string(),
        max_tokens: 8000,
    };

    let mut ctx = Context {
        tool_executor: Box::new(DefaultToolExecutor::new()),
        messages: vec![
            Message::SystemMessage("You are a helpful assistant".to_string()),
            Message::UserMessage(UserMessage::Text("Hello!".to_string())),
        ],
    };

    base_loop(
        &mut ctx,
        &conf,
        |model, base_url, api_key, max_token, messages, chunk_reciever| {
            let model = model.to_string();
            let base_url = base_url.to_string();
            let api_key = api_key.to_string();
            let messages = messages.clone();
            let cr = chunk_reciever.clone();

            async move {
                let m = openai_compatible_stream(
                    &model, &base_url, &api_key, max_token, &messages, &cr,
                )
                .await
                .unwrap();

                Ok(m)
            }
        },
        on_chunk,
    )
    .await
    .unwrap();

    // println!("{:?}", ctx.messages.last());
}
