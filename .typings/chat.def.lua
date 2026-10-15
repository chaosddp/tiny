---@class RequestOptions
---@field url string
---@field body string
---@field headers table<string, string>
local RequestOptions = {}

---@class ChatOptions
---@field model string @name of the model
---@field base_url string
---@field api_key string
---@field stream? boolean
---@field max_tokens? integer
local ChatOptions = {}