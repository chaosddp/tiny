pub mod chat;
pub mod decision;
pub mod error;

pub type TinyResult<T> = Result<T, error::Error>;