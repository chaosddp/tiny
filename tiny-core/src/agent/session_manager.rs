use crate::TinyResult;
use mlua::prelude::*;

pub struct SessionManager {
    inner: LuaTable,
}

impl SessionManager {
    pub fn new(inner: LuaTable) -> Self {
        Self { inner }
    }

    pub fn set_current(&mut self, folder: &str) -> TinyResult<()> {
        // load session from specified folder, it will clean current state first
        todo!()
    }

    pub fn new_session(&mut self, name: &str) -> TinyResult<String> {
        // create a new session
        todo!()
    }
}
