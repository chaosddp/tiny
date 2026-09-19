---@class tiny
---@field conf  fun(t: TinyConfiguration): void                       @function to provide configuration
---@field tools table<string, fun(args?: table<string, any>): string> @tool collection for tool executor to search tool function by name
local Tiny = nil

---@type tiny
tiny = {}

---@class ChatOptions
---@field provider         string                                                                       @not supported yet, now only support openai compatible providers
---@field model            string                                                                       @model name to use
---@field base_url         string                                                                       @base url of the model provider
---@field api_key          string                                                                       @api key from provider
---@field max_tokens       integer                                                                      @max tokens for each session, default is 8000
---@field reasoning_effort "low" | "medium" | "high" | "customized_effort_name_from_different_provider" @effort when reasoning, default is low
---@field thinking         ThinkingOptions                                                              @options of thinking
local ChatOptions = nil

---@class ThinkingOptions
---@field type          "enabled" | "disabled" | "adaptive" | "customized_type_from_different_provider" @ type of thinking, default is "enabled"
---@field budget_tokens integer                                                                         @tokens for thinking
local ThinkingOptions = nil

---@class ToolParameter
---@field type     string  @type of the parameter
---@field desc     string  @description of the paremeter
---@field required boolean @if the parameter is required, default is false
local ToolParameter = nil

---@class Tool
---@field desc       string                       @description of the tool
---@field parameters table<string, ToolParameter> @parameters of this tool, key if the name of parameter, value if the parameter information
local Tool = nil

---@class TinyConfiguration
---@field chat           ChatOptions
---@field tools          table<string, Tool>
---@field chunk_receiver IChunkReceiver?
---@field tool_executor  IToolExecutor?
local TinyConfiguration = nil

---@class ToolCall
---@field name      string
---@field id        string
---@field index     integer
---@field arguments table<string, any> | string | nil
local ToolCall = nil

---@class MessageChunk
---@field content           string?
---@field reasoning_content string?
---@field tool_calls        ToolCall[]
---@field tool_result       string?
local MessageChunk = nil

---@class IChunkReceiver
---@field chunk fun(self, chunk: MessageChunk): void
local IChunkReceiver = nil

---@class IToolExecutor
---@field exec fun(self, name: string, args?: table<string, any>): string
local IToolExecutor = nil
