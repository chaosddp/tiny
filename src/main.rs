mod app;
mod config;
mod lua;

use clap::{Parser, Subcommand, ValueEnum};
use mlua::prelude::*;

use crate::app::{ChatOptions, TinyApplication};

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

    /// allow all the tools of current round
    #[arg(short, long)]
    allow: bool,

    /// do not memory about this chat
    #[arg(short, long)]
    no_memory: bool,

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
        command: SessionCommands,
    },
    /// Lua mode
    Lua {
        /// file to launch
        #[arg(short, long)]
        file: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
enum SessionCommands {
    /// show session content
    Show {
        /// name of the session
        #[arg(short, long)]
        name: String,
    },
    /// list all the sessions
    List,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let app = TinyApplication::new()?;
    app.load()?;

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Lua { file, args }) => {
            app.script(&file, args)?;
        }
        Some(Commands::Init {
            tools: _,
            extensions: _,
            model: _,
        }) => {
            println!("init");
        }
        Some(Commands::Session { command }) => match command {
            SessionCommands::List => {
                app.session("list", LuaValue::Nil)?;
            }
            SessionCommands::Show { name } => {
                app.session("show", name)?;
            }
        },
        None => {
            let user_message = cli.content.join(" ");

            let mut options = ChatOptions::default();
            options.allow = cli.allow;
            options.image = cli.image;
            options.model = cli.model;
            options.no_memory = cli.no_memory;
            options.session = cli.session;

            app.chat(&user_message, &options)?;
        }
    }

    Ok(())
}
