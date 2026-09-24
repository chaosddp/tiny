---@class JsonWrapper
local JsonWrapper = {}

--- load and parse string into lua table
---@param s string
---@return any
function JsonWrapper.load(s) end

--- dump lua object into json string
---@param o       table<string, any>
---@param pretty? boolean
---@return string
function JsonWrapper.dump(o, pretty) end

---@type JsonWrapper
json = {}
