

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
end

--- handle chunks if enbled stream mode
function tiny.chunk(...)

end