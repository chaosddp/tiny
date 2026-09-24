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

function _m:chunk(chunk)
    if self.reasoning and chunk.reasoning and #chunk.reasoning > 0 then
        if self.state ~= ChunkState.Reasoning then
            print("\n\n[Assistant - Reasoning]\n\n")
        end

        self.state = ChunkState.Reasoning

        io.write(chunk.reasoning)
        io.flush()
    end

    if chunk.content and #chunk.content > 0 then
        if self.state ~= ChunkState.Content then
            print("\n\n[Assistant - Content]\n\n")
        end

        self.state = ChunkState.Content

        io.write(chunk.content)
        io.flush()
    end

    if chunk["end"] then
        self.state = ChunkState.Completed
        print("\n")
    end
end

function _m:message(message)
    if self.reasoning and message.reasoning and #message.reasoning > 0 then
       print("\n\n[Assistant - Reasoning]\n\n")
       print(message.reasoning) 
    end

    if message.content and #message.content > 0 then
       print("\n\n[Assistant - Content]\n\n")
       print(message.content)     end
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
