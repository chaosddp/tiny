pub mod agent;
pub mod chat;
pub mod decision;
pub mod error;
pub mod lua;

pub type TinyResult<T> = Result<T, error::Error>;
