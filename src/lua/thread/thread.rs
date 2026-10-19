use std::{
    fs::File,
    io::Read,
    path::Path,
    sync::Arc,
    thread::{self, JoinHandle},
};

use log::debug;
use mlua::prelude::*;

use crate::lua::{
    register_all,
    thread::{
        registry::{LuaChannel, LuaChannelRegistry},
        value::LuaThreadValue,
    },
};

pub struct LuaThread {
    handler: JoinHandle<()>,
}

impl LuaThread {
    pub fn new(
        content: String,
        args: Vec<LuaThreadValue>,
        channel_registry: Arc<LuaChannelRegistry>,
    ) -> Self {
        let registry = channel_registry.clone();
        let handler = thread::spawn(move || {
            let lua = Lua::new();
            let globals = lua.globals();

            // here without thread
            register_all(&lua).expect("Fail to register utilities");

            // register the get channel method
            let channel_table = lua.create_table().unwrap();

            channel_table
                .set(
                    "get",
                    lua.create_function(move |_: &Lua, name: String| {
                        let channel = registry.get_or_create(name.as_ref());

                        Ok(LuaChannel(channel))
                    })
                    .unwrap(),
                )
                .unwrap();

            let tiny_table: LuaTable = lua.create_table().unwrap();

            tiny_table.set("channel", channel_table).unwrap();

            globals.set("tiny", tiny_table).unwrap();

            globals.set("arg", args).expect("Fail to send arguments");

            if let Err(e) = lua.load(content).exec() {
                panic!("Fail to execute thread code: {}", e.to_string());
            }
        });

        Self { handler }
    }

    pub fn is_finished(&self) -> bool {
        self.handler.is_finished()
    }
}

impl LuaUserData for LuaThread {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("is_finished", |_, thread, ()| Ok(thread.is_finished()));
    }
}

pub fn register(lua: &Lua, registry: Arc<LuaChannelRegistry>) -> LuaResult<()> {
    let globals = lua.globals();

    let thread_table = lua.create_table()?;
    let channel_table = lua.create_table()?;

    let channel_registry = registry.clone();
    channel_table.set(
        "get",
        lua.create_function(move |lua: &Lua, name: String| {
            let channel = channel_registry.get_or_create(name.as_ref());

            lua.create_userdata(LuaChannel(channel))
        })?,
    )?;

    let thread_registry = registry.clone();
    thread_table.set(
        "newThread",
        lua.create_function(
            move |_, (file_or_content, args): (String, LuaVariadic<LuaThreadValue>)| {
                let file_path = Path::new(&file_or_content);

                if file_path.exists() && file_path.is_file() {
                    let mut scripts = String::new();

                    {
                        let mut fp = File::open(file_path)?;
                        fp.read_to_string(&mut scripts)?;
                    }

                    debug!("start thread from script content: {0}", file_or_content);

                    Ok(LuaThread::new(
                        scripts,
                        args.to_vec(),
                        thread_registry.clone(),
                    ))
                } else {
                    debug!("start thread from script content: {0}", file_or_content);

                    Ok(LuaThread::new(
                        file_or_content,
                        args.to_vec(),
                        thread_registry.clone(),
                    ))
                }
            },
        )?,
    )?;

    let tiny_table: LuaTable = globals.get("tiny")?;

    tiny_table.set("thread", thread_table)?;
    tiny_table.set("channel", channel_table)?;

    Ok(())
}
