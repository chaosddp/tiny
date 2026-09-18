pub mod types;

#[cfg(feature = "async")]
pub mod async_impl;

#[cfg(feature = "sync")]
pub mod sync_impl;

#[allow(unused_imports)]

pub mod prelude {
    #[cfg(feature = "sync")]
    pub use super::sync_impl::{ChatClient, ChunkReceiver, ToolExecutor, tiny_loop};
    pub use super::types::{Message, MessageChunk, UserMessage};
}

#[allow(unused_imports)]
pub use types::*;

#[allow(unused_imports)]
#[cfg(feature = "sync")]
pub use sync_impl::*;

#[allow(unused_imports)]
#[cfg(feature = "async")]
pub use async_impl::*;
