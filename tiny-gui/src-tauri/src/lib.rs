use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

use tauri::ipc::Channel;
use tauri::{Manager, State};
use tiny_core::agent::builder::TinyAgentBuilder;
use tiny_core::core::agent::ChatOptions;
use tiny_core::core::chat::chunk::Chunk;
use tiny_core::core::chat::messages::UserMessage;
use tiny_core::core::TinyResult;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn chat(
    state: State<'_, AppContext>,
    message: String,
    on_chunk: Channel<Chunk>,
) -> tauri::Result<()> {
    state.chat(&message, on_chunk).expect("Fail to call chat");

    Ok(())
}

#[allow(dead_code)]
struct AppContext {
    handler: JoinHandle<TinyResult<()>>,
    message_sender: Sender<(UserMessage, Channel<Chunk>)>,
}

impl AppContext {
    pub fn new() -> Self {
        let (message_sender, message_receiver) = mpsc::channel();

        let handler = thread::spawn(move || {
            let exec_path = std::env::current_exe().unwrap();
            let mut exec_folder = exec_path.parent().unwrap();

            #[cfg(debug_assertions)]
            {
                exec_folder = exec_folder // debug
                    .parent() // target
                    .unwrap()
                    .parent() // workspace
                    .unwrap();
            }

            let mut agent = TinyAgentBuilder::new(&exec_folder.to_path_buf())?
                .config(std::env::current_dir()?.parent().unwrap().parent().unwrap().join(".tiny.json"))?
                .load_exetensions()?
                .load_tools(exec_folder.to_path_buf())?
                .build()?;

            let options = ChatOptions {
                model: "qwen3.5".into(),
                base_url: "http://localhost:11434/v1".into(),
                api_key: "ollama".into(),
                stream: Some(true),
                stream_include_usage: None,
                max_tokens: Some(64000),
                reasoning_effort: None,
            };

            loop {
                let (user_message, channel): (UserMessage, Channel<Chunk>) =
                    message_receiver.recv()?;

                agent.chat("openai", user_message, &options, |chunk| {
                    // TODO: sometime here will cause 'send to closed channel' error
                    channel.send(chunk).unwrap();
                    Ok(())
                })?;
            }

            // Ok(())
            // unreachable!()
        });

        Self {
            handler,
            message_sender,
        }
    }

    pub fn chat(&self, message: &str, channel: Channel<Chunk>) -> TinyResult<()> {
        self.message_sender
            .send((UserMessage::Text(message.into()), channel))
            .expect("Fail to send message, may be the background thread failed to start.");

        Ok(())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppContext::new());

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![chat])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
