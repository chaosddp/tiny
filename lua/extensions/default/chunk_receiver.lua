local io = io
local setmetatable = setmetatable

---@class DefaultChunkReceiver: IChunkReceiver
---@field private current_chunk_id string
---@field private reasoning        boolean
local _m = {}

function _m:chunk(chunk)
    if self.reasoning and chunk.reasoning and #chunk.reasoning > 0 then
        if self.current_chunk_id ~= chunk.id then
            print("\n\n[Assistant - Reasoning]\n\n")

            self.current_chunk_id = chunk.id
        end

        io.write(chunk.reasoning)
        io.flush()
    end

    if chunk.content and #chunk.content > 0 then
        if self.current_chunk_id ~= chunk.id then
            print("\n\n[Assistant - Content]\n\n")

            self.current_chunk_id = chunk.id
        end

        io.write(chunk.content)
        io.flush()
    end
end

function _m:message(message)
    if self.reasoning and message.reasoning and #message.reasoning > 0 then
        print("\n\n[Assistant - Reasoning]\n\n")
        print(message.reasoning)
    end

    if message.content and #message.content > 0 then
        print("\n\n[Assistant - Content]\n\n")
        print(message.content)
    end
end

function _m:show_reasoning(b)
    self.reasoning = b
end

---@return IChunkReceiver
local function ChunkReceiver()
    return setmetatable({
        reasoning = true,
        current_chunk_id = nil
    }, { __index = _m })
end

return ChunkReceiver
