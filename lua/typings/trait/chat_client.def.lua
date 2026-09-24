---@alias ThinkingType "enabled" | "disabled" | "adaptive" | string
---@alias ReasoningEffort "low" | "medium" | "high" | string

--- Options for chatting
---@class ChatOptions
---@field model                 string          @model name
---@field base_url              string
---@field api_key               string
---@field provider?             string          @llm provider name, this will be used for error handing and request processing
---@field stream?               boolean         @if enable stream mode, default is true
---@field stream_include_usage? boolean         @if include tokens usage in stream mode, default is true
---@field max_tokens?           integer         @max tokens, default is 8000
---@field reasoning_effort?     ReasoningEffort @reasoning effort, default is "medium"
---@field thinking_type?        ThinkingType    @thinking type, default is 'enabled'
---@field thinking_budget?      integer         @thining budget, default is 8192 if thiking is not disabled
---@field support_tools?        boolean         @if the model support tools?, default is true
---@field support_thinking?     boolean         @if the model support thining, default is true
---@field support_vision?       boolean         @if the model support vision, default is true
local ChatOptions = {}

--- Chat client used to chat with llm with related provider
---@class IChatClient
local IChatClient = {}

--- Start a chat for current provider
---
--- NOTE:
---
--- 1. If do not provide chunk_receiver, it wil just return an AssistantMessage
---
--- 2. If stream=false, then chunk_receiver will not be called.
---
---@param messages        Message[]
---@param options         ChatOptions
---@param tools?          Tool[]
---@param chunk_receiver? IChunkReceiver
---@return AssistantMessage
function IChatClient:chat(messages, options, tools, chunk_receiver) end

---@class RequestOptions
---@field url     string
---@field body    string
---@field headers table<string, string>
local RequestOptions = {}

--- process differences for proiders
---@class ChatProvider
local ChatProvider = {}

--- Prepare request options for current chat provider
---@param messages Message[]
---@param options  ChatOptions
---@param tools?   Tool[]
---@return boolean
---@return RequestOptions
function ChatProvider:request(messages, options, tools) end

--- Parse and convert provider SSE chunk into general one, it is called when stream=true
---@param chunk_str string
---@return Chunk
function ChatProvider:chunk(chunk_str) end

--- Parse the content from llm server into AssistantMessage, this is called when stream=false
---@param message_str string
---@return bool                      @if success
---@return AssistantMessage | string @AssistantMessage if succes, else error message
function ChatProvider:message(message_str) end

--- Create a new ChatClient for a provider
---@param provider ChatProvider
---@return IChatClient
function ChatClient(provider) end
