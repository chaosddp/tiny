---@class AgentLoopContext
---@field messages Message[]
---@field options  ChatOptions
---@field tools?   Tool[]
local AgentLoopContext = {}

---@class AgentLoopExtensions
---@field chat_client     IChatClient
---@field tool_executor?  IToolExecutor
---@field chunk_receiver? IChunkReceiver
local AgentLoopExtensions = {}

---@alias AgentLoop fun(ctx: AgentLoopContext, extensions: AgentLoopExtensions): void
