use crate::TinyResult;
use mlua::prelude::*;
pub struct ToolExecutor {
    inner: LuaTable,
}

impl ToolExecutor {
    pub fn new(inner: LuaTable) -> Self {
        Self { inner }
    }

    pub fn load(&self, folder: &str) -> TinyResult<()> {
        todo!()
    }

    pub fn execute(&self, name: &str, arguments: Option<&str>) -> TinyResult<String> {
        todo!()
    }
}
