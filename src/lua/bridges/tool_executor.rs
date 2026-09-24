use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

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

    pub fn load(&mut self, path: PathBuf) -> LuaResult<()> {
        debug!("Loading tools from: {:?}", path);

        if let Ok(entrys) = glob(&format!("{}/*", path.to_str().unwrap())) {
            for entry in entrys {
                match entry {
                    Ok(p) => {
                        debug!("Try to load tools from: {:?}", p);

                        if p.is_dir() {
                            let info_file = p.join("tool.json");
                            let tool_file = p.join("tool.lua");

                            if info_file.exists()
                                && tool_file.exists()
                                && info_file.is_file()
                                && tool_file.is_file()
                            {
                                let tool_name = p.file_name().unwrap().to_str().unwrap();

                                let mut script_buf = String::new();

                                {
                                    let mut fp = File::open(tool_file)?;

                                    fp.read_to_string(&mut script_buf)?;
                                }

                                let tool_func: LuaFunction = self.lua.load(script_buf).eval()?;

                                self.tool_functions.insert(tool_name.to_string(), tool_func);
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

    fn init(&self) -> LuaResult<()> {
        // register our extensions
        crate::lua::utils::prelude::register_all(&self.lua)?;

        Ok(())
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

        methods.add_method_mut("load", |_, executor, path: String| {
            Ok(executor.load(Path::new(&path).to_path_buf()))
        });
    }
}

pub fn register(lua: &Lua) -> TinyResult<()> {
    let globals = lua.globals();

    globals.set(
        "DefaultToolExecutor",
        lua.create_function(|_, ()| {
            let mut executor = LuaToolExecutor::new();

            executor.init()?;

            // load modules unter tiny/tools
            let exe_path = std::env::current_exe()?;
            let tools_dir = exe_path.parent().unwrap().join("lua").join("tools");

            executor.load(tools_dir)?;

            Ok(executor)
        })?,
    )?;

    Ok(())
}
