use std::{cell::RefCell, io::Write, ops::Deref};

use clap::{Parser, Subcommand, ValueEnum};
use tiny_core::{agent::agent::TinyAgent, core::agent::ChatOptions};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ThiinkingEffort {
    Low,
    Medium,
    High,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// specify model to use
    #[arg(short, long)]
    model: Option<String>,

    /// chat in specified session, use default session if not specified
    #[arg(short, long)]
    session: Option<String>,

    /// images to use in the chat, can be an image list seperated with ','
    #[arg(short, long)]
    image: Option<String>,

    /// thinking effort, default is medium
    #[arg(short, long)]
    thinking_effort: Option<ThiinkingEffort>,

    /// show reasoning content
    #[arg(short, long)]
    reasoning: bool,

    /// allow all the tools of current round
    #[arg(short, long)]
    allow: bool,

    /// show tokens usage after response
    #[arg(short, long)]
    usage: bool,

    /// message chat with model
    #[arg(trailing_var_arg = true)]
    content: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// init current project with tiny
    Init {
        /// tools for current project
        #[arg(short, long)]
        tools: Option<Vec<String>>,
        /// extensions for current project
        #[arg(short, long)]
        extensions: Option<Vec<String>>,
        /// default model for current project
        #[arg(short, long)]
        model: Option<String>,
    },
    /// manage sessions for current project
    Session {
        #[command(subcommand)]
        command: Option<SessionCommands>,
    },
}

#[derive(Subcommand)]
enum SessionCommands {
    /// create a new session - backup and clear default one
    New,
    /// show session content
    Show {
        /// name of the session
        #[arg(short, long)]
        name: String,
    },
    /// list all the sessions
    List,
}

enum ChunkingState {
    None,
    Content,
    Reasoning,
}

impl Default for ChunkingState {
    fn default() -> Self {
        ChunkingState::None
    }
}

#[derive(Default)]
struct ConsoleChunkReceiver {
    id: RefCell<Option<String>>,
    state: RefCell<ChunkingState>,
}

impl ConsoleChunkReceiver {
    pub fn new() -> Self {
        Default::default()
    }
}

impl ConsoleChunkReceiver {
    fn recv(&self, chunk: tiny_core::core::chat::chunk::Chunk) -> tiny_core::core::TinyResult<()> {
        let mut id = self.id.borrow_mut();

        match id.deref() {
            Some(nid) => {
                if nid != &chunk.id {
                    std::io::stdout().write("\n\n".as_bytes())?;
                    std::io::stdout().flush()?;

                    *id = Some(chunk.id);
                }
            }
            None => {
                *id = Some(chunk.id);
            }
        }

        if let Some(content) = chunk.content
            && content.len() > 0
        {
            let mut state = self.state.borrow_mut();

            match *state {
                ChunkingState::Content => {}
                _ => {
                    std::io::stdout().write("\n\n[ Assistant ]\n\n".as_bytes())?;
                    std::io::stdout().flush()?;

                    *state = ChunkingState::Content;
                }
            }

            std::io::stdout().write(content.as_bytes())?;
            std::io::stdout().flush()?;
        }

        if let Some(reasoning) = chunk.reasoning
            && reasoning.len() > 0
        {
            let mut state = self.state.borrow_mut();

            match *state {
                ChunkingState::Reasoning => {}
                _ => {
                    std::io::stdout().write("[ Reasoning ]\n\n".as_bytes())?;
                    std::io::stdout().flush()?;

                    *state = ChunkingState::Reasoning;
                }
            }

            std::io::stdout().write(reasoning.as_bytes())?;
            std::io::stdout().flush()?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init {
            tools: _,
            extensions: _,
            model: _,
        }) => {
            println!("init");
        }
        Some(Commands::Session { command }) => match command {
            Some(SessionCommands::New) => {}
            Some(SessionCommands::List) => {}
            Some(SessionCommands::Show { name: _ }) => {}
            None => {}
        },
        None => {
            println!("chat with content: {:}", cli.content.join(" "));
        }
    }

    let options = ChatOptions {
        model: "qwen3.5".into(),
        base_url: "http://localhost:11434/v1".into(),
        api_key: "ollama".into(),
        stream: Some(true),
        stream_include_usage: None,
        max_tokens: Some(64000),
        reasoning_effort: None,
    };

    let chunk_receiver = ConsoleChunkReceiver::new();

    let mut agent = TinyAgent::new()?;

    agent.chat("openai", "who are you?".into(), &options, |chunk| {
        chunk_receiver.recv(chunk)?;

        Ok(())
    })?;

    Ok(())
}
