---@class ModelFeatures
---@field vision  boolean
---@field thining boolean
---@field tools   boolean
local ModelFeatures = {}

---@class Model
---@field model       string
---@field provider    string
---@field base_url    string
---@field api_key     string
---@field max_tokens? integer
---@field features    ModelFeatures
local Model = {}

---@class Configurations
---@field default_model string
---@field models        table<string, Model>
---@field tools?        string[]
---@field extensions?   string[]
local Configurations = {}

---@class UserOptions
---@field image?     string
---@field allow?     boolean
---@field no_memory? boolean
---@field model      string
---@field session?   string
local UserOptions = {}

tiny = {}

--- called before started, pass current configurations
---@param t Configurations
function tiny.load(t) end

--- handle 'session' command
---@param command  "list" | "show" @command to execute
---@param options? any
function tiny.session(command, options) end

--- handle default chat command
---@param content string
---@param options UserOptions
function tiny.chat(content, options) end

--- handle chunks if enbled stream mode
---@param chunk Chunk
function tiny.chunk(chunk) end

--- message from anywhere
---@param message Message
function tiny.message(message) end

--- handle error that cause application crash
function tiny.error(...) end
