use crate::core::TinyResult;
use crate::lua::agent::TinyAgent;
mod core;
mod lua;

fn run() -> TinyResult<()> {
    env_logger::init();

    let agent = TinyAgent::new();

    agent.init()?;

    agent.run()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match run() {
        Err(e) => panic!("{}", e.to_string()),
        Ok(_) => {}
    }

    Ok(())
}
