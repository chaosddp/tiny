local require = require
local ipairs = ipairs
local type = type
local pcall = pcall

---@class Extensions
---@field chat_clients    table<string, IChatClient>
---@field chunk_receiver? IChunkReceiver
---@field tool_executor?  IToolExecutor
---@field input_source?   IInputSource
local loaded_extensions = {
    chat_clients = {},
    chunk_receiver = nil,
    tool_executor = nil,
    input_source = nil
}

---@class ExtensionRegisterContext
local ExtensionRegisterContext = {}

---@param name   string
---@param client IChatClient
function ExtensionRegisterContext.add_chat_client(name, client)
    ---@diagnostic disable-next-line: unnecessary-if
    if name and client then
        loaded_extensions.chat_clients[name] = client
    end
end

---@param executor? IToolExecutor
function ExtensionRegisterContext.set_tool_executor(executor)
    loaded_extensions.tool_executor = executor
end

---@param receiver IChunkReceiver
function ExtensionRegisterContext.set_chunk_recever(receiver)
    loaded_extensions.chunk_receiver = receiver
end

---@param source IInputSource
function ExtensionRegisterContext.set_input_source(source)
    loaded_extensions.input_source = source
end

---@class ExtensionManager
local ExtensionManager = {}

--- load extension from 'extensions' folder by name
---@param configs    table<string, any>
---@param extensions table<string, any>
function ExtensionManager:load(configs, extensions)
    if not extensions or #extensions == 0 then return end

    for _, ext_name in ipairs(extensions) do
        local extension_module_path = "extensions." .. ext_name .. ".extension"

        local success, register_func = pcall(require, extension_module_path)

        if success and type(register_func) == "function" then
            local ext_config = nil

            ---@diagnostic disable-next-line: unnecessary-if
            if configs then
                local all_ext_configs = configs.extensions

                if all_ext_configs then
                    ext_config = all_ext_configs[ext_name]
                end
            end

            register_func(ExtensionRegisterContext, ext_config)
        else
            print("fail to load extension: " .. ext_name .. ", message: " .. register_func)
        end
    end

    return loaded_extensions
end

return ExtensionManager
