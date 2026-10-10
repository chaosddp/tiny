use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use glob::glob;
use log::debug;
use mlua::prelude::*;

use crate::core::chat::tool::Tool;

pub struct LuaToolExecutor {
    lua: Lua,
    tool_functions: HashMap<String, LuaFunction>,
}

impl LuaToolExecutor {
    pub fn new(tools_root: &str) -> LuaResult<Self> {
        let lua = Lua::new();
        let globals = lua.globals();
        let package: LuaTable = globals.get("package")?;

        package.set("path", format!("{0}/?.lua;{0}/?/init.lua", tools_root))?;

        crate::lua::register_all(&lua)?;

        Ok(Self {
            lua,
            tool_functions: HashMap::new(),
        })
    }

    pub fn load(&mut self, path: PathBuf) -> LuaResult<Vec<Tool>> {
        debug!("Loading tools from: {:?}", path);

        let mut tool_definitions = vec![];

        // try to load directly
        let info_file = path.join("tool.json");
        let tool_file = path.join("tool.lua");

        if info_file.exists() && tool_file.exists() {
            let tool_name = path.file_name().unwrap().to_str().unwrap();

            if let Ok(tool) = self.load_tool(tool_name, tool_file, info_file) {
                debug!("Loaded tool: {}", tool_name);

                tool_definitions.push(tool);
            } else {
                debug!("Failed to load tool: {}", tool_name);
            }
        }

        // load collections
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

                                if let Ok(tool) = self.load_tool(tool_name, tool_file, info_file) {
                                    debug!("Loaded tool: {}", tool_name);

                                    tool_definitions.push(tool);
                                } else {
                                    debug!("Failed to load tool: {}", tool_name);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(tool_definitions)
    }

    pub fn execute(&self, name: &str, parameters: Option<String>) -> LuaResult<String> {
        // TODO: pcall to make sure the function will not stop the lua state
        match self.tool_functions.get(name) {
            Some(tool_func) => Ok(tool_func.call::<String>(parameters)?),
            _ => Err(LuaError::RuntimeError(format!("Invalid tool: {}", name))),
        }
    }

    fn load_tool(
        &mut self,
        tool_name: &str,
        tool_file: PathBuf,
        info_file: PathBuf,
    ) -> LuaResult<Tool> {
        let mut script_buf = String::new();

        {
            let mut fp = File::open(tool_file)?;

            fp.read_to_string(&mut script_buf)?;
        }

        let tool_func: LuaFunction = self.lua.load(script_buf).eval()?;

        self.tool_functions.insert(tool_name.to_string(), tool_func);

        let info = serde_json::from_reader(File::open(info_file)?)
            .map_err(|e| mlua::Error::DeserializeError(e.to_string()))?;

        Ok(info)
    }
}
