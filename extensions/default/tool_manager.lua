local base_dir = path.source_base_dir()

local search_path = string.format(
    "%s;%s/tiny/?.lua;%s/tiny/?/init.lua;%s/tools/?.lua;%s/tools/?/init.lua",
    package.path,
    base_dir,
    base_dir,
    base_dir,
    base_dir
)

local env = {}
for k, v in pairs(_G) do
    if k == "tiny" then
        -- skip tiny table and package table
    else
        env[k] = v
    end
end

env.tiny = {
    tools = {},
    funcs = {}
}

local tool_root_path = string.format("%s/tools", base_dir)

_ENV = env

package.path = search_path

---@class ToolManager: IToolManager
local ToolManager = {}

ToolManager.__index = ToolManager

---@return ToolManager
function ToolManager.new()
    return setmetatable({}, ToolManager)
end

function ToolManager:load(name)
    local tool_file = string.format("%s/tool.lua", tool_root_path)

    logger.debug("loading tool " .. name)

    -- check if it is a tool or tool collection
    if not path.exists(tool_file) then
        -- tool collection
        logger.debug("loading tool collection " .. name)

        local glob_pattern = string.format("%s/%s/*", tool_root_path, name)
        local entities = path.glob(glob_pattern)

        ---@diagnostic disable-next-line: param-type-mismatch
        for _, folder in ipairs(entities) do
            if path.is_dir(folder) then
                local tool_file = string.format("%s/tool.lua", folder)

                if path.exists(tool_file) then
                    local path_info = path.get_path_info(folder)
                    local tool = require(name .. "." .. path_info.base_name .. ".tool")

                    logger.debug("loaded tool " .. tool.definition.name)

                    table.insert(tiny.tools, tool.definition)
                    tiny.funcs[tool.definition.name] = tool.func
                end
            end
        end
    else
        -- tool file
        logger.debug("loading tool " .. name)

        local tool = require(name .. ".tool")

        table.insert(tiny.tools, tool.definition)
        tiny.funcs[tool.definition.name] = tool.func
    end
end

function ToolManager:call(tool, arguments)
    if tool == nil then
        return "invalid tool name"
    end

    local tool_func = tiny.funcs[tool]

    if tool_func then
        local success, ret = pcall(tool_func, arguments)

        if success then
            return ret
        else
            return "fail to call tool: " .. ret
        end
    end

    return "cannot find tool: " .. tool
end

function ToolManager:tools()
    return tiny.tools
end

return ToolManager
