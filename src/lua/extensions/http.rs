use std::io::BufRead;
use std::io::BufReader;

use mlua::UserData;
use mlua::prelude::*;
use reqwest::blocking::{Client, ClientBuilder};

use super::json::lua_to_json_value;
use crate::core::TinyResult;

pub struct LuaHttpClient {
    client: Client,
}

impl UserData for LuaHttpClient {
    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "post",
            |lua, client, (url, data, cb, headers): (String, LuaValue, LuaFunction, Option<LuaTable>)| {
                let mut request_builder = client.client.post(url);

                match data {
                    LuaValue::Table(t) => {
                        let v = lua_to_json_value(lua, LuaValue::Table(t));

                        request_builder = request_builder.json(&v);
                    }
                    LuaValue::String(s) => {
                        let s_bytes = s.as_bytes();
                        let s_str = String::from_utf8_lossy(&s_bytes).to_string();
                        request_builder = request_builder.body(s_str);
                    }
                    _ => {
                        return Err(mlua::Error::RuntimeError(
                            "Invalid request data".to_string(),
                        ));
                    }
                }

                if let Some(header_table) = headers {
                    for pair in header_table.pairs::<String, String>() {
                        if let Ok((k, v)) = pair {
                            request_builder = request_builder.header(k, v);
                        }
                    }
                }

                match request_builder.send() {
                    Ok(resp) => {
                        let reader = BufReader::new(resp);

                        for line_ret in reader.lines() {
                            match line_ret {
                                Ok(line)=> {
                                    cb.call::<()>(line)?;
                                }
                                _=> {}
                            }
                        }

                        Ok(())
                    }
                    Err(e) => Err(mlua::Error::RuntimeError(e.to_string())),
                }
            },
        )
    }
}

pub fn register(lua: &Lua) -> TinyResult<()> {
    let globals = lua.globals();

    globals.set(
        "HttpClient",
        lua.create_function(|_, ()| {
            let client = ClientBuilder::new().build().unwrap();

            Ok(LuaHttpClient { client })
        })?,
    )?;

    Ok(())
}
