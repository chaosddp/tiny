---@class OpenAIUsage
---@field prompt_tokens     integer
---@field completion_tokens integer
---@field total_tokens      integer
local OpenAIUsage = {}

--- NOTE: definition is not completed
---@class OpenAIMessage
---@field role          string
---@field content       string
---@field reasoning?    string
local OpenAIMessage = {}

---@class OpenAIChoice
---@field index   integer
---@field message OpenAIMessage
---@field finish_reason string
local OpenAIChoice = {}

---@class OpenAIMessageRespons
---@field id                 string
---@field object             string
---@field created            integer
---@field model              string
---@field system_fingerprint string
---@field choices            OpenAIChoice[]
---@field usage?             OpenAIUsage
local OpenAIMessageRespons = {}
