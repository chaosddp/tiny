use log::debug;
use mlua::prelude::*;

pub struct EventHub {
    inner: LuaTable,
}

impl EventHub {
    pub fn new(inner: LuaTable) -> Self {
        Self { inner }
    }

    pub fn send<T>(&self, event: &str, payload: T)
    where
        T: IntoLua,
    {
        if let Err(error) = self.inner.call_method::<()>("sub", payload) {
            debug!(
                "Fail to sub event {} to lua side: {}",
                event,
                error.to_string()
            );
        }
    }
}
