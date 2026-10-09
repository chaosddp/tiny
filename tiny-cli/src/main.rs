use clap::{Parser, Subcommand, ValueEnum};
use tiny_core::agent::agent::TinyAgent;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // let cli = Cli::parse();

    // match cli.command {
    //     Some(Commands::Init {
    //         tools: _,
    //         extensions: _,
    //         model: _,
    //     }) => {
    //         println!("init");
    //     }
    //     Some(Commands::Session { command }) => match command {
    //         Some(SessionCommands::New) => {}
    //         Some(SessionCommands::List) => {}
    //         Some(SessionCommands::Show { name: _ }) => {}
    //         None => {}
    //     },
    //     None => {
    //         println!("chat with content: {:}", cli.content.join(" "));
    //     }
    // }

    // let lua = Lua::new();
    let agent = TinyAgent::new()?;

    agent.run()?;

    Ok(())
}
