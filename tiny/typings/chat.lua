---@class HttpError
---@field status_code? integer
---@field message? string

---@class ChatOptions
---@field model      string
---@field api_key    string
---@field base_url   string
---@field max_tokens integer
---@field stream     boolean
local ChatOptions = {}

---@class ChatClient
local ChatClient = {}

---@param messages        Message[]
---@param options         ChatOptions
---@param tools?          Tool[]
---@param chunk_callback? fun(chunk: Chunk): void
---@return boolean
---@return AssistantMessage | HttpError | nil
function ChatClient:chat(messages, options, tools, chunk_callback) end
