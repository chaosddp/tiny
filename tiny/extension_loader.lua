---@class ExtensionLoader
local ExtensionLoader = {}

ExtensionLoader.__index = ExtensionLoader

---@return ExtensionLoader
function ExtensionLoader.new()
    if tiny.extensions == nil then
        tiny.extensions = {
            ---@type table<string, ChatClient>
            chat_clients = {},
            ---@type IToolManager?
            tool_manager = nil,
            ---@type fun(messages: Message[], options: ChatOptions)?
            chat_loop = nil,
            ---@type ISessionManager?
            session_manager = nil,
        }
    end

    local builtin_path = string.format("%s/extensions", path.source_base_dir())

    -- add the folder as search path
    local old_package_path = package.path

    package.path = table.concat({
        old_package_path,
        builtin_path .. "/?.lua",
        builtin_path .. "/?/init.lua"
    }, ";")

    return setmetatable({}, ExtensionLoader)
end

-- TODO: can i make it a sandbox?
---@param name string
function ExtensionLoader:load(name)
    require(name .. ".extension")
end

return ExtensionLoader
