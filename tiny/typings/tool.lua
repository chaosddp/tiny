---@class ToolManager
local ToolManager = {}

--- load tools from folder
---@param folder
function ToolManager:load(folder) end

---@param tool       string @tool name to call
---@param arguments? string @to call the tool
---@return string @result or error message of tool call
function ToolManager:call(tool, arguments) end

---@return Tool[]
function ToolManager:tools() end