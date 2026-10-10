use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use glob::glob;
use log::debug;
use mlua::prelude::*;

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

    pub fn execute(&self, name: &str, parameters: Option<String>) -> LuaResult<String> {
        // TODO: pcall to make sure the function will not stop the lua state
        match self.tool_functions.get(name) {
            Some(tool_func) => Ok(tool_func.call::<String>(parameters)?),
            _ => Err(LuaError::RuntimeError(format!("Invalid tool: {}", name))),
        }
    }

    fn init(&self) -> LuaResult<()> {
        // register our utils
        crate::lua::register_all(&self.lua)?;

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
