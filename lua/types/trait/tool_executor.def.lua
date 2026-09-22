---@class IToolExecutor
local IToolExecutor = {}

---Execute a tool function
---@param name       string
---@param id         string
---@param parameters string?
---@return string
function IToolExecutor:execute(name, id, parameters) end

--- Load tools from specified folder
---@param path string @path to tools to load
function IToolExecutor:load(path) end

--- Create a new default tool execute, that will run tools in a seperated lua state
---@return IToolExecutor
function DefaultToolExecutor() end
