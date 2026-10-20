---@class ToolManager: IToolManager
---@field private lua_state LuaState
---@field private _tools Tool[] @tools cache
local ToolManager = {}

ToolManager.__index = ToolManager

---@return ToolManager
function ToolManager.new()
    local lua_state = lua.newState()
    local base_dir = path.source_base_dir()
    base_dir = string.gsub(base_dir, "\\", "/")

    local search_path = string.format(
        "%s/tiny/?.lua;%s/tiny/?/init.lua;%s/tools/?.lua;%s/tools/?/init.lua",
        base_dir,
        base_dir,
        base_dir,
        base_dir
    )

    lua_state:execute(string.format([[tool_root_path = "%s/tools"]], base_dir))

    lua_state:execute(string.format([[package.path = "%s"]], search_path))

    lua_state:execute([[require "utils"]])

    lua_state:execute(
        [[
tiny = {
    tools = {},
    funcs = {},
}

function tiny.load_tool(name)
    local tool_file = string.format("%s/tool.lua", tool_root_path)

    logger.debug("loading tool " .. name)

    -- check if it is a tool or tool collection
    if not path.exists(tool_file) then
        -- tool collection
        logger.debug("loading tool collection " .. name)

        for _, folder in ipairs(path.glob(string.format("%s/%s/*", tool_root_path, name))) do
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

function tiny.execute_tool(tool_name, tool_arguments)
    if tool_name == nil then
        return "invalid tool name"
    end

    local tool_func = tiny.funcs[tool_name]

    if tool_func then
        return tool_func(tool_arguments)
    end

    return "cannot find tool: " .. tool_name
end
    ]]
    )

    return setmetatable({
        lua_state = lua_state,
        _tools = nil
    }, ToolManager)
end

function ToolManager:load(name)
    local load_script = string.format("tiny.load_tool('%s')", name)

    self.lua_state:eval(load_script)
end

function ToolManager:call(tool, arguments)
    local ret = self.lua_state:eval(
        string.format("tiny.execute_tool('%s', %s)", tool, arguments == nil and "nil" or arguments)
    )

    ---@cast ret string
    return ret
end

function ToolManager:tools()
    if self._tools ~= nil then
       return self._tools 
    end

    local tools = self.lua_state:eval("tiny.tools")

    self._tools = tools

    ---@diagnostic disable-next-line: return-type-mismatch
    return tools
end

return ToolManager
