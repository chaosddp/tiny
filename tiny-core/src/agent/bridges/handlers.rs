use crate::core::{TinyResult, agent::EventHandler, chat::messages::ChatMessage};
use mlua::prelude::*;

pub struct LuaEventHandlers {
    lua: WeakLua,
    handlers: LuaTable,
}

impl LuaEventHandlers {
    pub fn new(lua: &Lua, handlers: LuaTable) -> LuaResult<Self> {
        Ok(Self {
            handlers,
            lua: lua.weak(),
        })
    }

    fn call(&self, event_name: &str, args: impl IntoLuaMulti + Clone) -> TinyResult<()> {
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

impl EventHandler for LuaEventHandlers {
    fn loop_start(&self) -> crate::core::TinyResult<()> {
        self.call("loop_start", ())?;

        Ok(())
    }

    fn loop_end(&self) -> TinyResult<()> {
        self.call("loop_end", ())?;

        Ok(())
    }

    fn loop_round(&self, n: u32) -> TinyResult<()> {
        self.call("loop_round", n)?;

        Ok(())
    }

    fn message(&self, message: &ChatMessage) -> TinyResult<()> {
        self.call("message", message.into_lua(&self.lua.upgrade())?)?;

        Ok(())
    }

    fn assistant_message(
        &self,
        message: &crate::core::chat::messages::AssistantMessage,
    ) -> TinyResult<()> {
        let msg = ChatMessage::Assistant(message.clone());

        self.message(&msg)?;

        Ok(())
    }

    fn user_message(&self, message: &crate::core::chat::messages::UserMessage) -> TinyResult<()> {
        let msg = ChatMessage::User(message.clone());

        self.message(&msg)
    }
}
