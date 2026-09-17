use crate::luaenv::lua::*;
use reqwest::{Client, StatusCode};

pub struct LuaHttpResponse {
    status: StatusCode,
    bytes: Option<Vec<u8>>,
}

impl LuaUserData for LuaHttpResponse {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("status", |_, response, ()| Ok(response.status.as_u16()));

        methods.add_method("text", |lua, response, ()| {
            if let Some(data) = &response.bytes {
                Ok(String::from_utf8_lossy(&data)
                    .to_string()
                    .into_lua(&lua)
                    .unwrap_or(LuaNil))
            } else {
                Ok(LuaNil)
            }
        });
    }
}

pub struct LuaHttpClient(Client);

impl LuaHttpClient {
    pub fn new() -> reqwest::Result<Self> {
        let client = Client::builder().build()?;

        Ok(LuaHttpClient(client))
    }
}

impl LuaUserData for LuaHttpClient {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method("get", |_, client, url: String| async move {
            let resp = client
                .0
                .get(url)
                .send()
                .await
                .map_err(|e| LuaError::RuntimeError(e.to_string()))?;

            Ok(LuaHttpResponse {
                status: resp.status(),
                bytes: if let Ok(data) = resp.bytes().await {
                    Some(data.to_vec())
                } else {
                    None
                },
            })
        });
    }
}
