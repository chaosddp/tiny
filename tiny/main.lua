require "utils"

local ExtensionLoader = require "extension_loader"

---@param config Configurations
function tiny.load(config)
    local extension_loader = ExtensionLoader.new()

    if config.extensions then
        for _, extension in ipairs(config.extensions) do
            extension_loader:load(extension)
        end
    end
end

--- handle 'session' command
function tiny.session(command, options)
end

--- handle default chat command
function tiny.chat(content, options)
    print(string.format("chat content: %s", content))

    -- local thread = tiny.thread.newThread(
    --     [[

    -- local n1, n2 = arg[1] or 1, arg[2] or 10
    -- local channel = tiny.channel.get("chat_channel")

    -- print(channel)

    -- for i=n1, n2 do
    --     channel:send(i)
    -- end

    -- ]]
    -- )

    -- local channel = tiny.channel.get("chat_channel")

    -- while not thread:is_finished() do
    --     local v = channel:recv("j")

    --     if v ~= nil then
    --         print(v)
    --     end
    -- end

    -- print(thread:is_finished())

    local openai = require "default.openai"

    local success, msg = openai:chat({
        {
            role = "system",
            content = "You are a helpful assistant"
        },
        {
            role = "user",
            content = "What can you do?"
        }
    },
        {
            model = "qwen3.5",
            base_url = "http://localhost:11434/v1",
            api_key = "ollama",
            max_tokens = 64000,
            stream = true
        }, nil, tiny.chunk)

    if not success then
        if msg ~= nil then
            print(msg.status_code)
            print(msg.message)
        end
    end

    print()
    if success and msg then
        print(msg.content)
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
