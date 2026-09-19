#[cfg(all(feature = "async", feature = "sync"))]
compile_error!("feature \"async\" and feature \"sync\" cannot be enabled at the same time");

#[cfg(feature = "async")]
compile_error!("feature \"async\" is not completed.");

mod agent;
mod core;
mod luaenv;
mod openai;

use crate::agent::sync_agent::{Image, TinyAgent};
use crate::openai::sync_impl::OpenaiClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut agent = TinyAgent::new(
        "You are a helpful assistant",
        ".",
        Box::new(OpenaiClient::new()),
        None,
    )?;

    agent.chat("what is the weather in Beijing.")?;

    Ok(())
}
