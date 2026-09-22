---@class IAgentLoop
local IAgentLoop = {}

---@param options        ChatOptions
---@param messages       Message[]
---@param chat_client    IChatClient
---@param chunk_receiver IChunkReceiver
---@param tool_executor  IToolExecutor
function IAgentLoop:loop(messages, options, chat_client, chunk_receiver, tool_executor) end
