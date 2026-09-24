---@type IInputSource
local DefaultInputSource = {}

function DefaultInputSource:input()
    io.write("\nInput you message: ")
    return io.read("l")
end

return DefaultInputSource
