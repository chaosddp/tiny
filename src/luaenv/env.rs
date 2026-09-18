use std::{fs::File, io::Read, path::Path};

use mlua::{MaybeSend, prelude::*};

/// Lua environment wrapper
#[allow(dead_code)]
pub struct LuaEnv {
    /// name of env, it will be used as root namespace name (lua table),
    /// so give it a reasonable name.
    name: String,
    /// the default lua vm we attach to
    lua: Lua,
}

impl LuaEnv {
    pub fn new(name: &str) -> LuaResult<Self> {
        let lua = unsafe { Lua::unsafe_new() };
        lua.globals().set(name, lua.create_table()?)?;

        // TODO: process the name, make sure it can be used as a global variable
        Ok(LuaEnv {
            name: name.to_string(),
            // we enable all the modules here, but we will provide an environment when loading any scripts.
            lua,
        })
    }
}

#[allow(dead_code)]
impl LuaEnv {
    pub fn weak(&self) -> WeakLua {
        self.lua.weak()
    }

    /// Add a named function to root of current env
    #[cfg(feature = "sync")]
    pub fn add_function<F, A, R>(&self, name: &str, func: F) -> LuaResult<()>
    where
        F: Fn(&Lua, A) -> LuaResult<R> + MaybeSend + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        self.add_member(name, self.lua.create_function(func)?)
    }

    #[cfg(feature = "async")]
    pub fn add_async_function<F, A, FR, R>(&self, name: &str, func: F) -> LuaResult<()>
    where
        F: Fn(Lua, A) -> FR + MaybeSend + 'static,
        A: FromLuaMulti,
        FR: Future<Output = LuaResult<R>> + MaybeSend + 'static,
        R: IntoLuaMulti,
    {
        self.add_member(name, self.lua.create_async_function(func)?)
    }

    pub fn add_member(&self, name: &str, member: impl IntoLua) -> LuaResult<()> {
        if !name.contains(".") {
            // just set by name, we do not care if name exist
            let root: LuaTable = self.lua.globals().get(self.name.as_str())?;

            root.set(name, member)?;
        } else {
            // assuming that we need to set by name in nested table
            // TODO: we need to make sure that the name is a valid variable name
            let parts: Vec<&str> = name.split(".").collect();

            let mut target_table: LuaTable = self.lua.globals().get(self.name.as_str())?;

            for pname in &parts[..parts.len() - 1] {
                if let Ok(pvalue) = target_table.get::<LuaValue>(*pname) {
                    // the name is exist, let's check if it is a table
                    match pvalue {
                        LuaValue::Table(sub_table) => target_table = sub_table,
                        LuaValue::Nil => {
                            target_table.set(*pname, self.lua.create_table()?)?;

                            target_table = target_table.get::<LuaTable>(*pname).unwrap();
                        }
                        _ => {
                            return Err(LuaError::RuntimeError(format!(
                                "{} is not a table to set",
                                *pname
                            )));
                        }
                    }
                }
            }

            target_table.set(*parts.last().unwrap(), member)?;
        }

        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn call_async<T>(&self, name: &str, args: impl IntoLuaMulti) -> LuaResult<T>
    where
        T: FromLuaMulti,
    {
        let v: LuaValue = self.lua.globals().get_path(name)?;

        match v {
            LuaValue::Function(func) => {
                return func.call_async::<T>(args).await;
            }
            _ => {}
        }

        Err(LuaError::RuntimeError(format!(
            "Fail to call: {}, not exist, or not callable.",
            name
        )))
    }

    #[cfg(feature = "sync")]
    pub fn call<T>(&self, name: &str, args: impl IntoLuaMulti) -> LuaResult<T>
    where
        T: FromLuaMulti,
    {
        let v: LuaValue = self.lua.globals().get_path(name)?;

        match v {
            LuaValue::Function(func) => {
                return func.call::<T>(args);
            }
            _ => {}
        }

        Err(LuaError::RuntimeError(format!(
            "Fail to call: {}, not exist, or not callable.",
            name
        )))
    }

    /// add a dir as package path, to support require
    pub fn add_package_path(&self, dir: &str) -> LuaResult<()> {
        let globals = self.lua.globals();

        let package: LuaTable = globals.get("package")?;

        let path: LuaString = package.get("path")?;

        package.set("path", path.to_str()?.to_string() + ";" + dir)?;

        Ok(())
    }

    /// load a script file
    #[cfg(feature = "async")]
    pub async fn exec_script_async(&self, file: &str) -> LuaResult<()> {
        let file_path = Path::new(file);

        if !file_path.exists() || !file_path.is_file() {
            return Err(LuaError::RuntimeError(format!(
                "file not exist, not invalid: {}",
                file
            )));
        }

        let mut script = String::new();

        {
            let mut fp = File::open(file_path)?;
            fp.read_to_string(&mut script)?;
        }

        self.lua.load(script).exec_async().await?;

        Ok(())
    }

    #[cfg(feature = "sync")]
    pub fn exec_script(&self, file: &str) -> LuaResult<()> {
        let file_path = Path::new(file);

        if !file_path.exists() || !file_path.is_file() {
            return Err(LuaError::RuntimeError(format!(
                "file not exist, not invalid: {}",
                file
            )));
        }

        let mut script = String::new();

        {
            let mut fp = File::open(file_path)?;
            fp.read_to_string(&mut script)?;
        }

        self.lua.load(script).exec()?;

        Ok(())
    }
}
