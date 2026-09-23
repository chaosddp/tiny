---@class IAgentLoop
local IAgentLoop = {}

---@param messages        Message[]
---@param options         ChatOptions
---@param chat_client     IChatClient
---@param tool_executor?  IToolExecutor
---@param chunk_receiver? IChunkReceiver
function IAgentLoop:loop(messages, options, chat_client, tool_executor, chunk_receiver) end
