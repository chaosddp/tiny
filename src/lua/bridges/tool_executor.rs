use std::{collections::HashMap, path::PathBuf};

use glob::glob;
use log::debug;
use mlua::prelude::*;

use crate::core::TinyResult;

/// Default tool execute that load and run all the tool functions in a seperated lua state with limit permissions
/// 
/// It will try to load builtin tools first, then load tools from specified folders
/// 
/// It will register a function 'DefaultToolExecutor' that return a new object. NOTE: each instance will own a new lua state!
/// 
/// # Example
/// 
/// ```lua
/// 
/// --- create a tool executor, it will load builtin tools by default 
/// local tool_executor = DefaultToolExecutor()
/// 
/// --- call a tool by name with id and parameters
/// print(tool_executor:execute("get_weather", "id", "BeiJing"))
/// 
/// ```
pub struct LuaToolExecutor {
    lua: Lua,
    tool_functions: HashMap<String, LuaFunction>,
}

impl LuaToolExecutor {
    pub fn new() -> Self {
        let lua = Lua::new();

        Self {
            lua,
            tool_functions: HashMap::new(),
        }
    }

    pub fn load(&mut self) -> LuaResult<()> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();

        // TODO: load tools from 3rd folders

        // register our extensions
        crate::lua::extensions::prelude::register_all(&self.lua)?;

        let globals = self.lua.globals();
        let require_func: LuaFunction = globals.get::<LuaFunction>("require")?;

        // load modules unter tiny/tools
        let tools_dir = exe_dir.join("tiny").join("tools");
        let tools_dir_str = tools_dir.to_str().unwrap();

        debug!("Loading tools from: {}", tools_dir_str);

        if let Ok(entrys) = glob(&format!("{}/*", tools_dir_str)) {
            for entry in entrys {
                match entry {
                    Ok(p) => {
                        debug!("Try to load tools from: {:?}", p);

                        if p.is_dir() {
                            let info_file = p.join("info.lua");
                            let tool_file = p.join("tool.lua");

                            if info_file.exists()
                                && tool_file.exists()
                                && info_file.is_file()
                                && tool_file.is_file()
                            {
                                let tool_name = p.file_name().unwrap().to_str().unwrap();

                                match require_func
                                    .call::<LuaFunction>(format!("tiny.tools.{}.tool", tool_name))
                                {
                                    Ok(tool_func) => {
                                        debug!("Loaded tool function: {}", &tool_name);

                                        self.tool_functions
                                            .insert(tool_name.to_string(), tool_func);
                                    }
                                    Err(e) => {
                                        debug!(
                                            "cannot load tool {}, error: {}",
                                            tool_name,
                                            e.to_string()
                                        );
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub fn execute(&self, name: &str, _: &str, parameters: Option<String>) -> LuaResult<String> {
        match self.tool_functions.get(name) {
            Some(tool_func) => Ok(tool_func.call::<String>(parameters)?),
            _ => Err(LuaError::RuntimeError(format!("Invalid tool: {}", name))),
        }
    }

    #[allow(dead_code)]
    fn add_package_path(&self, dir: PathBuf) -> LuaResult<()> {
        let globals = self.lua.globals();

        let package: LuaTable = globals.get("package")?;
        let path: LuaString = package.get("path")?;

        package.set(
            "path",
            path.to_str()?.to_string() + ";" + dir.to_str().unwrap(),
        )?;

        Ok(())
    }
}

impl LuaUserData for LuaToolExecutor {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "execute",
            |_, executor, (name, id, parameters): (String, String, Option<String>)| {
                Ok(executor.execute(&name, &id, parameters))
            },
        );
    }
}

pub fn register(lua: &Lua) -> TinyResult<()> {
    let globals = lua.globals();

    globals.set(
        "DefaultToolExecutor",
        lua.create_function(|_, ()| {
            let mut executor = LuaToolExecutor::new();

            executor.load()?;

            Ok(executor)
        })?,
    )?;

    Ok(())
}
