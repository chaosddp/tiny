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

    -- load tools
    if tiny.extensions.tool_manager and tiny.config.tools then
        for _, tool in ipairs(tiny.config.tools) do
            tiny.extensions.tool_manager:load(tool)
        end
    end
end

--- handle 'session' command
function tiny.session(command, options)
    if tiny.extensions.session_manager then
        if command == "list" then
            local default_session = tiny.extensions.session_manager:get_default()
            local sessions = tiny.extensions.session_manager:get_sessions()

            for _, session in ipairs(sessions) do
                if session == default_session then
                    print(session .. " - default")
                else
                    print(session)
                end
            end
        end
    end
end

--- handle default chat command
---@param content string
---@param options UserOptions
function tiny.chat(content, options)
    --- TODO: support prompt updating later
    ---@type Message[]
    local messages = {
        {
            role = "system",
            content = "You are a helpful assistant"
        }
    }

    if tiny.extensions.session_manager then
        tiny.extensions.session_manager:switch_session(options.session)

        local memories = tiny.extensions.session_manager:load_memory(options.session)

        for _, m in ipairs(memories) do
            table.insert(messages, m)
        end
    end

    ---@type UserMessage
    local user_message = {
        role = 'user',
        content = content
    }

    tiny.message(user_message)

    table.insert(messages, user_message)

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
    -- NOTE: we saved assistant message reasoning content here, but our chat client will ignore it while construct the request palyload
    if tiny.extensions.session_manager then
        tiny.extensions.session_manager:log_message(message)
    end
end
