---@class ExtensionLoader
local ExtensionLoader = {}

ExtensionLoader.__index = ExtensionLoader

---@return ExtensionLoader
function ExtensionLoader.new()
    if tiny.extensions == nil then
        tiny.extensions = {
            ---@type table<string, ChatClient>
            chat_clients = {},
            ---@type ToolManager?
            tool_executor = nil,
            ---@type fun(messages: Message[], options: ChatOptions): (bool, AssistantMessage)?
            chat_loop = nil
        }
    end

    return setmetatable({}, ExtensionLoader)
end

-- TODO: can i make it a sandbox?
function ExtensionLoader:load(folder)
    if not path.exists(folder) then
        return
    end

    -- add the folder as search path
    local old_package_path = package.path

    package.path = table.concat({
        old_package_path,
        folder .. "/?.lua",
        folder .. "/?/init.lua"
    }, ";")

    require "extension"

    package.path = old_package_path
end

return ExtensionLoader
