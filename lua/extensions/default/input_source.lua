---@type IInputSource
local DefaultInputSource = {}

function DefaultInputSource:input()
    io.write("\n\n[User]\n\n")
    return io.read("l")
end

return DefaultInputSource
