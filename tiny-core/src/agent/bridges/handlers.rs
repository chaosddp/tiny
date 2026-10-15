use crate::core::{TinyResult, agent::EventHandler};
use mlua::prelude::*;

pub struct LuaEventHandlers {
    handlers: LuaTable,
}

impl LuaEventHandlers {
    pub fn new(handlers: LuaTable) -> Self {
        Self { handlers }
    }
}

impl EventHandler for LuaEventHandlers {
    fn emit<T>(&self, event_name: &str, args: &T) -> TinyResult<()>
    where
        T: IntoLua + Clone,
    {
        for cb_obj in self.handlers.sequence_values::<LuaTable>() {
            if let Ok(obj) = cb_obj {
                if let Ok(cb_func) = obj.get::<LuaFunction>(event_name) {
                    let is_stop = cb_func.call::<Option<bool>>((obj, args.clone()))?;

                    if let Some(stop) = is_stop
                        && stop
                    {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
