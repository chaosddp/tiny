-- called before started, pass current configurations
function tiny.conf(t)
    print(t.default_model)

    for k, _ in pairs(t.models) do
        print(k)
    end
end

--- called after 'tiny.conf' to do initialization
function tiny.load()
end

--- handle 'session' command
function tiny.session(command, options)
end

--- handle default chat command
function tiny.chat(content, options)
    print(string.format("chat content: %s", content))

    local thread = tiny.thread.newThread(
        [[

    local n1, n2 = arg[1] or 1, arg[2] or 10
    local channel = tiny.channel.get("chat_channel")

    print(channel)

    for i=n1, n2 do
        channel:send(i)
    end

    ]])

    local channel = tiny.channel.get("chat_channel")

    while not thread:is_finished() do
        local v = channel:recv("j")

        if v ~= nil then
            print(v)
        end
    end

    print(thread:is_finished())
end

--- handle chunks if enbled stream mode
function tiny.chunk(...)
end
