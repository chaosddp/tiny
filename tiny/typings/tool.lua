---@class IToolManager
local IToolManager = {}

--- load tools from folder
---@param folder string
function IToolManager:load(folder) end

---@param tool       string @tool name to call
---@param arguments? string @to call the tool
---@return string @result or error message of tool call
function IToolManager:call(tool, arguments) end

---@return Tool[]
function IToolManager:tools() end