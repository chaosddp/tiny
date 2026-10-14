use tiny_core::agent::agent::TinyAgent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = TinyAgent::new()?;

    agent.run()?;

    Ok(())
}
