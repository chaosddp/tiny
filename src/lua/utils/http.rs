use std::io::BufRead;
use std::io::BufReader;

use mlua::UserData;
use mlua::prelude::*;
use reqwest::Method;
use reqwest::blocking::{Client, ClientBuilder};

use super::json::lua_to_json_value;

fn send_http_request(
    client: &LuaHttpClient,
    method: Method,
    lua: &Lua,
    url: &str,
    data: Option<LuaValue>,
    headers: Option<LuaTable>,
    callback: Option<LuaFunction>,
) -> LuaResult<Option<LuaValue>> {
    #[allow(unused)]
    let mut result = None;

    let mut request_builder = client.client.request(method, url);

    if let Some(t_data) = data {
        match t_data {
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
                    "Invalid request data, it should be a table or string".to_string(),
                ));
            }
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
        Ok(response) => match response.error_for_status() {
            Ok(resp) => {
                let content_type = resp.headers().get("Content-Type").unwrap();

                if content_type == "text/event-stream" {
                    let reader = BufReader::new(resp);
                    let result_table = lua.create_table()?;

                    for line_ret in reader.lines() {
                        match line_ret {
                            Ok(line) => {
                                if callback.is_some() {
                                    callback.as_ref().unwrap().call::<()>(line.to_string())?;
                                }

                                result_table.push(line)?;
                            }
                            _ => {}
                        }
                    }

                    // for SSE stream, we return a list of lines
                    result = Some(LuaValue::Table(result_table));
                } else {
                    // for normal response, we return a byte string
                    let resp_bytes = resp
                        .bytes()
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;

                    result = Some(LuaValue::String(lua.create_string(resp_bytes)?));
                }
            }
            Err(e) => {
                return Err(mlua::Error::RuntimeError(e.to_string()));
            }
        },
        Err(e) => {
            return Err(mlua::Error::RuntimeError(e.to_string()));
        }
    }

    Ok(result)
}

pub struct LuaHttpClient {
    client: Client,
}

impl UserData for LuaHttpClient {
    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "send",
            |lua,
             client,
             (method_str, url, data, headers, callback): (
                String,
                String,
                Option<LuaValue>,
                Option<LuaTable>,
                Option<LuaFunction>,
            )| {
                let method = match method_str.to_lowercase().as_ref() {
                    "post" => Method::POST,
                    "get" => Method::GET,
                    "put" => Method::PUT,
                    "delete" => Method::DELETE,
                    _ => Method::GET,
                };

                send_http_request(client, method, lua, &url, data, headers, callback)
            },
        )
    }
}

pub fn register(lua: &Lua) -> LuaResult<()> {
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
