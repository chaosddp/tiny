pub mod error;
pub mod types;

pub type TinyResult<T> = Result<T, error::Error>;

#[cfg(feature = "sync")]
pub mod sync_impl;
