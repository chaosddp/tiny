pub mod error;

pub type TinyResult<T> = Result<T, error::Error>;
