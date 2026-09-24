---@class Json
local Json = {}

--- load and parse string into lua table
---@param s string
---@return any
function Json.load(s) end

--- dump lua object into json string
---@param o       table<string, any>
---@param pretty? boolean
---@return string
function Json.dump(o, pretty) end

---@type Json
json = {}
