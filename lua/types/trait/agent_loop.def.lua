---@class AgentLoopContext
---@field messages Message[]
---@field options  ChatOptions
---@field tools?   Tool[]
local AgentLoopContext = {}

---@class AgentLoopPlugins
---@field chat_client     IChatClient
---@field tool_executor?  IToolExecutor
---@field chunk_receiver? IChunkReceiver
local AgentLoopPlugins = {}

---@alias AgentLoop fun(ctx: AgentLoopContext, plugins: AgentLoopPlugins): void
