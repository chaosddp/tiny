---@class Chunk
---@field content?       string
---@field reasoning?     string
---@field tool_call?     ToolCall
---@field finish_reason? string
---@field tool_result?   string
---@field user_message?  string
local Chunk = {}

---@class IChunkReceiver
local IChunkReceiver = {}

---@param chunk Chunk
function IChunkReceiver:receive(chunk) end
