require "utils"

local ExtensionLoader = require "extension_loader"

function tiny.load()
    local extension_loader = ExtensionLoader.new()

    if tiny.config.extensions then
        for _, extension in ipairs(tiny.config.extensions) do
            extension_loader:load(extension)
        end
    end
end

--- handle 'session' command
function tiny.session(command, options)
end

--- handle default chat command
---@param content string
---@param options UserOptions
function tiny.chat(content, options)
    -- TODO: load message here
    ---@type Message[]
    local messages = {
        {
            role = "system",
            content = "You are a helpfule assistant"
        },
        {
            role = 'user',
            content = content
        }
    }

    local model_name = options.model or tiny.config.default_model

    local model = tiny.config.models[model_name]

    ---@type ChatOptions
    local chat_options = {
        model = model.model,
        api_key = model.api_key,
        base_url = model.base_url,
        stream = true,
        max_tokens = model.max_tokens or 8000,
        client = model.client
    }

    if tiny.extensions.chat_loop then
        tiny.extensions.chat_loop(messages, chat_options)
    end
end

--- handle chunks if enbled stream mode
function tiny.chunk(chunk)
    if chunk.content then
        io.write(chunk.content)
        io.flush()
    end

    if chunk.reasoning then
        io.write(chunk.reasoning)
        io.flush()
    end
end

function tiny.message(message)
    if message then
        print(message.role)
        print(message.content)
    end
end
