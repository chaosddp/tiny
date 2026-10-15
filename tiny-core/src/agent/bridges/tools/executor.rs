use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use glob::glob;
use log::debug;
use mlua::prelude::*;

use crate::core::chat::tool::Tool;

pub struct LuaToolExecutor {
    lua: Lua,
    tool_functions: HashMap<String, LuaFunction>,
    tools: Vec<Tool>,
}

impl LuaToolExecutor {
    pub fn from_tools(
        core_root: &PathBuf,
        tools_root: &PathBuf,
        tools: &Vec<String>,
    ) -> LuaResult<Self> {
        let lua = Lua::new();
        let globals = lua.globals();
        let package: LuaTable = globals.get("package")?;

        package.set(
            "path",
            format!(
                "{0}/?.lua;{0}/?/init.lua;{1}/?.lua;{1}/?/init.lua",
                core_root.to_string_lossy(),
                tools_root.to_string_lossy()
            ),
        )?;

        // patch the require function to adjust the core module
        lua.load(r#"
            local orig_require = require

            require = function (module)
                if string.sub(module, 1, 5) == "core." then
                    return orig_require(string.sub(module, 6))
                else
                    return orig_require(module)
                end
            end
        "#).exec()?;

        let require = globals.get::<LuaFunction>("require")?;

        require.call::<()>("init")?;

        crate::lua::register_all(&lua)?;

        let mut executor = Self {
            lua,
            tool_functions: HashMap::new(),
            tools: vec![],
        };

        for tool in tools {
            let tool_definitions = executor.load(tools_root.join(tool))?;

            for tool_def in tool_definitions {
                executor.tools.push(tool_def);
            }
        }

        Ok(executor)
    }

    pub fn get_tools(&self) -> Vec<Tool> {
        self.tools.clone()
    }

    pub fn execute(&self, name: &str, parameters: Option<String>) -> LuaResult<String> {
        match self.tool_functions.get(name) {
            Some(tool_func) => {
                let pcall = self.lua.globals().get::<LuaFunction>("pcall")?;

                let (success, result) = pcall.call::<(bool, String)>((tool_func, parameters))?;

                if success {
                    Ok(result)
                } else {
                    Err(LuaError::RuntimeError(result))
                }
            }
            _ => Err(LuaError::RuntimeError(format!("Invalid tool: {}", name))),
        }
    }

    fn load(&mut self, path: PathBuf) -> LuaResult<Vec<Tool>> {
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
