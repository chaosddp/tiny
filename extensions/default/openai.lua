local OpenAIChatBackend = {}

OpenAIChatBackend.__index = OpenAIChatBackend

function OpenAIChatBackend.new()
    return setmetatable({}, OpenAIChatBackend)
end

return OpenAIChatBackend
