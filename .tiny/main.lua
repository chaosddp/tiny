--- implementation part
local tiny = tiny
local tools = require "tools"

--- Our customize tool executor, it will loop for tool function from global object tiny.tools
---@class ToolExecutor: IToolExecutor
local ToolExector = {}

--- Execute tool function with arguments
---@param name      string              @name of the tool, we also use it as tool function name
---@param tool_args table<string, any>? @arguments to call the tool function
function ToolExector:exec(name, tool_args)
    local func = tiny.tools[name]

    if func ~= nil then
        return func(tool_args)
    end

    return "tool [" .. name .. "] not available"
end

--- Current chunk state, we use to determine how to insert a new line
---@enum ChunkState
local ChunkState = {
    NotStarted = 1,
    Reasoning = 2,
    Content = 3,
    ToolCall = 4
}

--- Our customize chunk receiver
---@class ChunkReceiver: IChunkReceiver
---@field state ChunkState
local ChunkReceiver = {
    state = ChunkState.NotStarted
}

function ChunkReceiver:chunk(chunk)
    if chunk.content ~= nil then
        if self.state ~= ChunkState.Content then
            self.state = ChunkState.Content

            io.write("\n\n[🤖] - [Assistant]\n\n")
        end

        io.write(chunk.content)
        io.flush()
    end

    if chunk.reasoning_content ~= nil then
        if self.state ~= ChunkState.Reasoning then
            self.state = ChunkState.Reasoning

            io.write("\n[🤔] - [Reasoning]\n\n")
        end

        io.write(chunk.reasoning_content)
        io.flush()
    end

    if chunk.tool_calls ~= nil then
        for _, tool_call in ipairs(chunk.tool_calls) do
            io.write("\n\n[🔧] - [Tool call(" .. tool_call.id .. ")]\n\n")

            io.write("name: " .. tool_call.name .. "\n\n")

            if tool_call.arguments ~= nil then
                -- TODO: we need a function to format table into pretty string
                io.write("arguments: " .. tool_call.arguments)
            end

            io.flush()
        end

        self.state = ChunkState.ToolCall
    end

    if chunk.tool_result ~= nil then
        io.write("\n\n[💬] - [Tool result]\n\n")

        io.write(chunk.tool_result)

        io.flush()

        self.state = ChunkState.ToolCall
    end
end

--- provide configurations for the application
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

    t.chunk_receiver = ChunkReceiver -- use customize chunk receiver instead default one
    t.tool_executor = ToolExector -- use customize tool executor instead default one

    -- add tool definitions from our module
    for _, tool in ipairs(tools) do
        t.tools[tool.name] = tool.definition
    end
end

-- attach tools to tiny.tools table, so that we can access it from the tool executor
for _, tool in ipairs(tools) do
    tiny.tools[tool.name] = tool.func
end

-- called each loop cycle, usage:
-- 1. cancel current loop
-- 2. retry from beginning
-- function tiny.on_inner_loop_count(n)
-- end

-- function tiny.on_response_error(e)
-- end

-- function tiny.before_chat(ctx)
-- end

-- function tiny.after_chat(ctx)
-- end
