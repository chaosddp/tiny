--- NOTE: definition is not completed
---@class OpenAIMessage
---@field role          string
---@field content       string
---@field reasoning?    string
---@field finish_reason string
local OpenAIMessage = {}

---@class OpenAIChoice
---@field index   integer
---@field message OpenAIMessage
local OpenAIChoice = {}

---@class OpenAIMessageRespons
---@field id                 string
---@field object             string
---@field created            integer
---@field model              string
---@field system_fingerprint string
---@field choices            OpenAIChoice[]
---@field usage              Usage
local OpenAIMessageRespons = {}
