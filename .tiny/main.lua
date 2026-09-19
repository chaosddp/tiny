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
---@field func       function                     @function to call
---@field parameters table<string, ToolParameter> @parameters of this tool, key if the name of parameter, value if the parameter information
local Tool = nil

---@class TinyConfiguration
---@field chat           ChatOptions
---@field tools          table<string, Tool>
---@field chunk_receiver IChunkReceiver?
local TinyConfiguration = nil

---@class ToolCall
---@field name      string
---@field id        string
---@field index     integer
---@field arguments table<string, any>?
local ToolCall = nil

---@class MessageChunk
---@field content           string?
---@field reasoning_content string?
---@field tool_call         ToolCall?
local MessageChunk = nil

---@class IChunkReceiver
---@field chunk fun(self, chunk: MessageChunk): void
local IChunkReceiver = nil

---@class IToolExecutor
---@field exec fun(self, tool_call: ToolCall): string @different with rust ToolExecutor, here we are expect a tool call object
local IToolExecutor = nil

---@class ToolExecutor: IToolExecutor
local ToolExector = {}

function ToolExector:exec(tool_call)
    return "not implemented"
end

---@enum ChunkState
local ChunkState = {
    NotStarted = 1,
    Reasoning = 2,
    Content = 3,
    ToolCall = 4
}

---@class ChunkReceiver: IChunkReceiver
---@field state ChunkState
local ChunkReceiver = {
    state = ChunkState.NotStarted
}

function ChunkReceiver:chunk(chunk)
    if chunk.content ~= nil then
        if self.state ~= ChunkState.Content then
            self.state = ChunkState.Content

            io.write("\n\n[Assistant]\n\n")
        end

        io.write(chunk.content)
        io.flush()
    end

    if chunk.reasoning_content ~= nil then
        if self.state ~= ChunkState.Reasoning then
            self.state = ChunkState.Reasoning

            io.write("\n[🤔Reasoning]\n\n")
        end

        io.write(chunk.reasoning_content)
        io.flush()
    end
end

function get_weather(o)
    return [[
        Condition: Cloudy
        Temperature: 17°C (feels comfortable/cool)
        Humidity: 85%
        Wind: East at 4 mph
        High Temperature: 23°C
        Low Temperature: 15°C
        Tonight: Temperatures will drop to around 15°C and 14°C later tonight.
    ]]
end

---@param t TinyConfiguration
function tiny.conf(t)
    t.chat.provider = "openai" -- use openai compatible provider
    t.chat.model = "qwen3.5"
    t.chat.base_url = "http://localhost:11434/v1"
    t.chat.api_key = "ollama"

    t.chat.max_tokens = 10240000

    t.chat.thinking.type = "disabled"
    t.chat.thinking.budget_tokens = 8192

    t.chat.reasoning_effort = "low" -- low, medium, hight, any other string

    t.chunk_receiver = ChunkReceiver

    t.tools = {
        get_weather = {
            desc = "get weather of specified city", -- descript of the tool
            func = get_weather,                     -- real function to call
            parameters = {
                city = {
                    ["type"] = "string",
                    desc = "city name",
                    required = true
                }
            }
        }
    }
end

-- called each loop cycle, usage:
-- 1. cancel current loop
-- 2. retry from beginning
function tiny.on_inner_loop_count(n)
end

function tiny.on_response_error(e)
end

function tiny.before_chat(ctx)
end

function tiny.after_chat(ctx)
end
