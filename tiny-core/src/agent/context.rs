use crate::agent::{
    chat_client::ChatClient, event_hub::EventHub, session_manager::SessionManager,
    tool_executor::ToolExecutor,
};
use mlua::prelude::*;

pub struct Context {
    pub event_hub: EventHub,
    pub chat_client: ChatClient,
    pub tool_executor: ToolExecutor,
    pub session_manager: SessionManager,
}

pub struct ContextBuilder(LuaTable);

impl ContextBuilder {
    pub fn new(lua: &Lua) -> LuaResult<Self> {
        Ok(Self(lua.create_table()?))
    }

    pub fn build(lua: &Lua, table: LuaTable) -> LuaResult<Context> {
        let session_manager_backend: LuaTable = table.get("session_manager")?;
        let tool_executor_backend: LuaTable = table.get("tool_executor")?;
        let event_hub_backend: LuaTable = table.get("event_hub")?;
        let chat_clients_table: LuaTable = table.get("chat_clients")?;

        Ok(Context {
            event_hub: EventHub::new(event_hub_backend),
            chat_client: ChatClient::new(lua, chat_clients_table)?,
            tool_executor: ToolExecutor::new(tool_executor_backend),
            session_manager: SessionManager::new(session_manager_backend),
        })
    }
}

impl LuaUserData for ContextBuilder {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("session_manager", |_, builder, tbl: LuaTable| {
            builder.0.set("session_manager", tbl)?;

            Ok(())
        });

        methods.add_method("tool_executor", |_, builder, tbl: LuaTable| {
            builder.0.set("tool_executor", tbl)?;

            Ok(())
        });

        methods.add_method("event_hub", |_, builder, tbl: LuaTable| {
            builder.0.set("event_hub", tbl)?;

            Ok(())
        });

        methods.add_method(
            "chat_client",
            |lua, builder, (name, tbl): (String, LuaTable)| {
                let chat_clients = builder
                    .0
                    .get::<LuaTable>("chat_clients")
                    .unwrap_or(lua.create_table()?);

                chat_clients.set(name, tbl)?;

                builder.0.set("chat_clients", chat_clients)?;

                Ok(())
            },
        );

        methods.add_method_once("collect", |_, builder, ()| Ok(builder.0))
    }
}
