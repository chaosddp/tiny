---@class ToolManager: IToolManager
---@field private tools      Tool[]
---@field private tools_func table<string, fun(arguments: any): string>
local ToolManager = {}

ToolManager.__index = ToolManager

---@return ToolManager
function ToolManager.new()
    return setmetatable({ tools = {}, tools_func = {} }, ToolManager)
end

function ToolManager:load(folder)
end

function ToolManager:call(tool, arguments)
end

function ToolManager:tools()
end

return ToolManager
