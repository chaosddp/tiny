local io = io
local setmetatable = setmetatable

---@enum ChunkState
local ChunkState = {
    NotStarted = 0,
    Reasoning = 1,
    Content = 2,
    Completed = 3
}

---@class DefaultChunkReceiver: IChunkReceiver
---@field private state     ChunkState
---@field private reasoning boolean
local _m = {}

function _m:receive(chunk)
    if self.reasoning and chunk.reasoning and #chunk.reasoning > 0 then
        if self.state ~= ChunkState.Reasoning then
            print("\n\n[Reasoning]\n\n")
        end

        self.state = ChunkState.Reasoning

        io.write(chunk.reasoning)
        io.flush()
    end

    if chunk.content and #chunk.content > 0 then
        if self.state ~= ChunkState.Content then
            print("\n\n[Assistant]\n\n")
        end

        self.state = ChunkState.Content

        io.write(chunk.content)
        io.flush()
    end
end

function _m:show_reasoning(b)
    self.reasoning = b
end

---@return IChunkReceiver
local function ChunkReceiver()
    return setmetatable({
        reasoning = true,
        state = ChunkState.NotStarted
    }, { __index = _m })
end

return ChunkReceiver
