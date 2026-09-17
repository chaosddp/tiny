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
---@field chat  ChatOptions
---@field tools table<string, Tool>
local TinyConfiguration = nil

function get_weather(city)
end

---@param t TinyConfiguration
function tiny.conf(t)
    t.chat.provider = "openai" -- use openai compatible provider
    t.chat.model = "qwen3.5"
    t.chat.base_url = "http://localhost:11434/v1"
    t.chat.api_key = "ollama"

    t.chat.max_tokens = 10240000

    t.chat.thinking.type = "enabled"
    t.chat.thinking.budget_tokens = 8192

    t.chat.reasoning_effort = "low" -- low, medium, hight, any other string

    -- t.system_prompt = "myprompt"

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
