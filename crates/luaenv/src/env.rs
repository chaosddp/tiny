use mlua::{MaybeSend, prelude::*};

/// Lua environment wrapper
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

impl LuaEnv {
    pub fn weak(&self) -> WeakLua {
        self.lua.weak()
    }

    /// Add a named function to root of current env
    pub fn add_function<F, A, R>(&self, name: &str, func: F) -> LuaResult<()>
    where
        F: Fn(&Lua, A) -> LuaResult<R> + MaybeSend + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        self.add_member(name, self.lua.create_function(func)?)
    }

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
}
