#[cfg(all(feature = "async", feature = "sync"))]
compile_error!("feature \"async\" and feature \"sync\" cannot be enabled at the same time");

#[cfg(feature = "async")]
compile_error!("feature \"async\" is not completed.");

mod core;
mod lua;
mod bridge;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    Ok(())
}
