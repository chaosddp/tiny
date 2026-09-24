---@diagnostic disable: missing-return
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

---@diagnostic disable-next-line: missing-fields
---@type Json
json = {}
