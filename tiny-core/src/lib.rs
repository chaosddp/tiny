pub type TinyResult<T> = Result<T, error::Error>;

pub mod error;
pub mod utils;
pub mod core;
pub mod agent;