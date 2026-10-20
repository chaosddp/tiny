local tiny = tiny
local io = io
local ipairs = ipairs

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

local chunk_state = 0

--- handle chunks if enbled stream mode
---@param chunk Chunk
function tiny.chunk(chunk)
    if chunk.reasoning and #chunk.reasoning > 0 then
        if chunk_state ~= 1 then
            print("-- Reasoning --\n\n")

            chunk_state = 1
        end

        io.write(chunk.reasoning)
        io.flush()
    end

    if chunk.content and #chunk.content > 0 then
        if chunk_state ~= 2 then
            print("\n\n-- Assistant --\n\n")

            chunk_state = 2
        end

        io.write(chunk.content)
        io.flush()
    end

    if chunk.done then
        print()

        chunk_state = 0
    end
end

function tiny.message(message)
    -- if message then
    --     print(message.role)
    --     print(message.content)
    -- end
end
