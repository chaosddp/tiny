// mod base;
mod core;
mod lua_helpers;

// use luaenv::lua::{Lua, LuaExternalResult, LuaResult};

// use crate::lua_helpers::http::{LuaHttpClient, LuaHttpResponse};

// fn t(_: &Lua, _: ()) -> LuaResult<()> {
//     println!("hello, world");

//     Ok(())
// }

// fn create_new_http_client(_: &Lua, _: ()) -> LuaResult<LuaHttpClient> {
//     let client = LuaHttpClient::new().into_lua_err()?;

//     Ok(client)
// }

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // let env = LuaEnv::new("tiny").unwrap();

    //     env.add_function("test", t).unwrap();
    //     env.add_function("a.b.test", t).unwrap();
    //     env.add_member("a.b.pi", 3.1415).unwrap();
    //     env.add_package_path("r/?.lua").unwrap();

    //     env.exec_script("t.lua").await.unwrap();

    //     env.call::<()>("print", ("hello", "-", "world"))
    //         .await
    //         .unwrap();

    //     let ret = env.call::<i32>("sum", (1, 2)).await.unwrap();

    //     println!("a + b = {}", ret);

    // env.add_function("http.newClient", create_new_http_client)
    //     .unwrap();

    // env.exec_script("t.lua").await.unwrap();
}
