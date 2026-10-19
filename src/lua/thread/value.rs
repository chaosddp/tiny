use mlua::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LuaThreadValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(Vec<u8>),
    Table(Vec<(LuaThreadValue, LuaThreadValue)>),
}

impl IntoLua for LuaThreadValue {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        Ok(match self {
            LuaThreadValue::Nil => LuaValue::Nil,
            LuaThreadValue::Boolean(v) => LuaValue::Boolean(v),
            LuaThreadValue::Integer(v) => LuaValue::Integer(v),
            LuaThreadValue::Number(v) => LuaValue::Number(v),
            LuaThreadValue::String(bytes) => LuaValue::String(lua.create_string(bytes)?),
            LuaThreadValue::Table(pairs) => {
                let table = lua.create_table()?;

                for (key, value) in pairs {
                    table.set(key.into_lua(lua)?, value.into_lua(lua)?)?;
                }

                LuaValue::Table(table)
            }
        })
    }
}

impl FromLua for LuaThreadValue {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        Ok(match value {
            LuaValue::Nil => LuaThreadValue::Nil,
            LuaValue::Boolean(v) => LuaThreadValue::Boolean(v),
            LuaValue::Integer(v) => LuaThreadValue::Integer(v),
            LuaValue::Number(v) => LuaThreadValue::Number(v),
            LuaValue::String(v) => LuaThreadValue::String(v.as_bytes().to_vec()),
            LuaValue::Table(tbl) => {
                let mut pairs = vec![];

                for pair in tbl.pairs::<LuaValue, LuaValue>() {
                    if let Ok((k, v)) = pair {
                        pairs.push((Self::from_lua(k, lua)?, Self::from_lua(v, lua)?));
                    }
                }

                LuaThreadValue::Table(pairs)
            }
            other => {
                return Err(LuaError::RuntimeError(
                    format! {"do not support {} between thread", other.type_name()},
                ));
            }
        })
    }
}
