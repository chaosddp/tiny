local io = io
local setmetatable = setmetatable

local _m = {}

function _m:receive(chunk)
    if chunk.reasoning then
        io.write(chunk.reasoning)
        io.flush()
    end

    if chunk.content then
        io.write(chunk.content)
        io.flush()
    end
end

---@return IChunkReceiver
local function ChunkReceiver()
    return setmetatable({}, { __index = _m })
end

return ChunkReceiver
