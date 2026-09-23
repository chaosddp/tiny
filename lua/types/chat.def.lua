---@alias MessageRole "system" | "user" | "assistant" | "tool" @type of the message

---@alias ImageDetail "auto" | "high" | "low" | string @image policy

---@alias FinishedReason "stop" | "length" | "tool_calls" | "content_filter" | string

---@class ToolCall
---@field id         string
---@field name       string
---@field index      integer
---@field arguments? string
local ToolCall = {}

---@class SystemMessage
---@field role    "system" @role of the mssage
---@field content string   @system prompt
local SystemMessage = {}

---@class ToolMessage
---@field role    "tool" @role of the message
---@field name    string @tool name
---@field id      string @tool call id
---@field content string @tool call result
local ToolMessage = {}

---@class Usage
---@field prompt_tokens     integer
---@field completion_tokens integer
---@field token_tokens      integer
local Usage = {}

---@class AssistantMessage
---@field role            "assistant"    @role of the message
---@field content         string         @content of the message
---@field reasoning?      string         @reasoning content of the message
---@field tool_calls?     ToolCall[]     @tool calls of the message
---@field finished_reason FinishedReason
---@field usage?          Usage
local AssistantMessage = {}

---@class TextUserMessage
---@field role    "user"
---@field content string
local TextUserMessage = {}

---@class ContentPart
---@field type          "text" | "image" | "video" | "file"
---@field text?         string
---@field image?        string
---@field image_detail? ImageDetail
---@field video?        string
---@field file?         string
local ContentPart = {}

---@class PartsUserMessage
---@field role  "user"
---@field parts ContentPart[]
local PartsUserMessage = {}

---@alias UserMessage TextUserMessage | PartsUserMessage @user mssage

---@alias Message SystemMessage | UserMessage | AssistantMessage | ToolMessage

---@class ToolParameter
---@field name        string
---@field type        string
---@field description string
---@field required?   boolean
local ToolParameter = {}

---@class Tool
---@field name        string
---@field description string
---@field parameters?  ToolParameter[]
local Tool = {}
