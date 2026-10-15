use crate::core::{ChatMessage, TinyResult};
use log::debug;
use mlua::{ObjectLike, prelude::LuaTable};

/// used to call lua backend to provide session management, it requrie the backend provide following methods:
///
/// 1. load_memory(self, session) ->ChatMessage[] : load memory from session
/// 2. new_session(self) -> string : create a new session
/// 3. switch_session(self, session) : switch to a new session
/// 4. last_session(self) -> string : get the latest used session
pub struct SessionManager {
    inner: LuaTable,
}

impl SessionManager {
    pub fn new(inner: LuaTable) -> Self {
        Self { inner }
    }

    /// load memory from session
    pub fn load_memory(&self, session: &str) -> TinyResult<Vec<ChatMessage>> {
        let messages = self.inner.call_method("load_memory", session)?;

        Ok(messages)
    }

    pub fn new_session(&self) -> TinyResult<String> {
        let session = self.inner.call_method("new_session", ())?;

        debug!("created new session: {}", session);

        Ok(session)
    }

    pub fn switch_session(&self, session: &str) -> TinyResult<()> {
        self.inner.call_method::<()>("switch_session", session)?;

        Ok(())
    }

    pub fn last_session(&self) -> TinyResult<Option<String>> {
        let session = self.inner.call_method("last_session", ())?;

        Ok(session)
    }
}
