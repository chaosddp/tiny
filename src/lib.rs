#[cfg(all(feature = "async", feature = "sync"))]
compile_error!("feature \"async\" and feature \"sync\" cannot be enabled at the same time");

#[cfg(feature = "async")]
compile_error!("feature \"async\" is not completed.");

pub mod luaenv;
pub mod core;
pub mod openai;
pub mod agent;