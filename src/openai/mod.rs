pub mod types;
pub mod utils;

#[cfg(feature = "async")]
pub mod async_impl;

#[cfg(feature = "sync")]
pub mod sync_impl;

#[cfg(feature = "async")]
use futures_util::StreamExt;
#[cfg(feature = "async")]
use reqwest::header;
#[cfg(feature = "async")]
use tokio::sync::mpsc;

#[allow(unused_imports)]
#[cfg(feature = "sync")]
pub use sync_impl::*;
