use mlua::prelude::*;

pub struct TinyApplication {
    // main thread for tiny
    lua: Lua,
}