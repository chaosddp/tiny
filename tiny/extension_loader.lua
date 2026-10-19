---@class ExtensionLoader
local ExtensionLoader = {}

ExtensionLoader.__index = ExtensionLoader

---@return ExtensionLoader
function ExtensionLoader.new()
    if tiny.extensions == nil then
        tiny.extensions = {
            chat_clients = {}
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
