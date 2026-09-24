use mlua::{IntoLua, Lua, Result as LuaResult, Value as LuaValue};
use serde_json::{Number, Value as JsonValue};

/// convert a Json value into Lua value recursively
pub fn json_to_lua(lua: &Lua, json_value: JsonValue) -> LuaValue {
    match json_value {
        JsonValue::Bool(v) => LuaValue::Boolean(v),
        JsonValue::Null => LuaValue::Nil,
        JsonValue::Number(n) => LuaValue::Number(n.as_f64().unwrap()),
        JsonValue::String(s) => s.into_lua(lua).unwrap(),
        JsonValue::Array(arr) => {
            // TODO: error handling?
            let lua_arr = lua.create_table().unwrap();

            for v in arr {
                lua_arr.push(json_to_lua(lua, v)).unwrap();
            }

            LuaValue::Table(lua_arr)
        }
        JsonValue::Object(o) => {
            let lua_obj = lua.create_table().unwrap();

            for (k, v) in o {
                lua_obj.set(k, json_to_lua(lua, v)).unwrap();
            }

            LuaValue::Table(lua_obj)
        }
    }
}

pub fn lua_to_json_value(lua: &Lua, lua_value: LuaValue) -> JsonValue {
    match lua_value {
        LuaValue::Boolean(b) => JsonValue::Bool(b),
        LuaValue::Integer(i) => JsonValue::Number(i.into()),
        LuaValue::Number(n) => JsonValue::Number(Number::from_f64(n).unwrap()),
        LuaValue::Nil => JsonValue::Null,
        LuaValue::String(s) => {
            let s_bytes = s.as_bytes();
            let s_str = String::from_utf8_lossy(&s_bytes);

            JsonValue::String(s_str.to_string())
        }
        LuaValue::Table(t) => {
            let mut sequnce_values = vec![];

            for v in t.sequence_values::<LuaValue>() {
                if let Ok(lv) = v {
                    sequnce_values.push(lua_to_json_value(lua, lv));
                }
            }

            if sequnce_values.len() > 0 {
                return JsonValue::Array(sequnce_values);
            }

            let mut kv_pairs: serde_json::Map<String, JsonValue> = serde_json::Map::new();

            for pair in t.pairs::<String, LuaValue>() {
                if let Ok((k, v)) = pair {
                    kv_pairs.insert(k, lua_to_json_value(lua, v));
                }
            }

            // if there is any kv pairs, then it will be json object, we will ignore sequential values
            return JsonValue::Object(kv_pairs);
        }
        _ => JsonValue::Null, // we do not support these types, so just return null
    }
}

pub fn lua_to_json_str(lua: &Lua, (value, pretty): (LuaValue, Option<bool>)) -> LuaResult<String> {
    let json_value = lua_to_json_value(lua, value);

    let json_str = match pretty {
        Some(p) => {
            if p {
                serde_json::to_string_pretty(&json_value)
                    .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
            } else {
                serde_json::to_string(&json_value)
                    .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
            }
        }
        None => serde_json::to_string(&json_value)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?,
    };

    Ok(json_str)
}

/// convert a Json value into Lua value recursively
pub fn parse_json(lua: &Lua, s: String) -> LuaResult<LuaValue> {
    let json_value: JsonValue =
        serde_json::from_str(&s).map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;

    Ok(json_to_lua(lua, json_value))
}

pub fn register(lua: &Lua) -> LuaResult<()> {
    let json_table = lua.create_table()?;

    json_table.set("load", lua.create_function(parse_json)?)?;
    json_table.set("dump", lua.create_function(lua_to_json_str)?)?;

    lua.globals().set("json", json_table)?;

    Ok(())
}
