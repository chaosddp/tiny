json = {}

--- load and parse string into lua table
---@param s string
---@return any
function json.load(s) end

--- dump lua object into json string
---@param o      table<string, any>
---@param pretty boolean?
---@return string
function json.dump(o, pretty) end
