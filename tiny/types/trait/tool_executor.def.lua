---@class IToolExecutor
local IToolExecutor = {}

---@param name       string
---@param id         string
---@param parameters string?
---@return string
function IToolExecutor:execute(name, id, parameters) end

---Create a new default tool execute, that will run tools in a seperated lua state
---@return IToolExecutor
function DefaultToolExecutor() end