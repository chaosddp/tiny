local table = table
local json = json
local pairs = pairs
local ipairs = ipairs

---@type ChatProvider
local OpenAIChatProvider = {}

---@param tool_call ToolCall
---@return table<string, any>
local function tool_call_to_openai(tool_call)
    return {
        id = tool_call.id,
        ["type"] = "function",
        ["function"] = {
            name = tool_call.name,
            arguments = tool_call.arguments
        }
    }
end

---@param message Message
---@return table<string, any>
local function message_to_openai(message)
    if message.role == "system" then
        ---@cast message SystemMessage
        return { role = "system", content = message.content }
    elseif message.role == "tool" then
        ---@cast message ToolMessage
        return { role = "tool", content = message.content, name = message.name }
    elseif message.role == "assistant" then
        ---@cast message AssistantMessage
        local openai_tool_calls = nil

        if message.tool_calls then
            openai_tool_calls = table.map(message.tool_calls, function (t)
                return tool_call_to_openai(t)
            end)
        end

        return {
            role = "assistant",
            content = message.content,
            reasoning_content = message.reasoning,
            reasoning_details = message.reasoning_details,
            tool_calls = openai_tool_calls
        }
    else
        local m = { role = "user" }

        if message.content then
            ---@cast message TextUserMessage
            m.content = message.content
        else
            m.parts = {}

            ---@cast message PartsUserMessage
            for _, part in ipairs(message.parts) do
                if part.type == "text" then
                    table.insert(m.parts, { ["type"] = "text", text = part.text })
                elseif part.type == "file" then
                    table.insert(m.parts, { ["type"] = "file_url", file_url = { url = part.file } })
                elseif part.type == "video" then
                    table.insert(m.parts, {
                        ["type"] = "video_url",
                        video_url = { url = part.video }
                    })
                else
                    if part.image and part.image then
                        local detail = part.image_detail or "auto"
                        table.insert(m.parts, {
                            ["type"] = "image_url",
                            image_url = { url = part.image, detail = detail }
                        })
                    end
                end
            end
        end

        return m
    end
end

---@param tool Tool
---@return table<string, any>
local function tool_to_openai(tool)
    local openai_tool_parameters = {
        type = "object",
        required = nil,
        parameters = nil
    }

    if tool.parameters and #tool.parameters > 0 then
        openai_tool_parameters.required = {}
        openai_tool_parameters.properties = {}

        for _, p in ipairs(tool.parameters) do
            if p.required then
                table.insert(openai_tool_parameters.required, p.name)
            end

            openai_tool_parameters.properties[p.name] = {
                type = p.type,
                description = p.description
            }
        end
    end

    return {
        type = "function",
        ["function"] = {
            name = tool.name,
            description = tool.description,
            parameters = openai_tool_parameters
        }
    }
end

function OpenAIChatProvider:request(messages, options, tools)
    local url = options.base_url .. "/chat/completions"

    local openai_tools = nil

    if tools and #tools > 0 then
        openai_tools = table.map(tools, function (t) return tool_to_openai(t) end)
    end

    local body = {
        model = options.model,
        messages = table.map(messages, function (m) return message_to_openai(m) end),
        tools = openai_tools,
        stream = options.stream or true,
        max_tokens = options.max_tokens or 8000
    }

    ---@type RequestOptions
    local request_options = {
        url = url,
        body = json.dump(body),
        headers = {
            Authorization = "Bearer " .. options.api_key,
            ["Content-Type"] = "application/json"
        }
    }

    return true, request_options
end

function OpenAIChatProvider:chunk(chunk_str)
    local chunk_message = json.load(chunk_str)

    ---@type Chunk
    local chunk = {}

    if chunk_message.choices and #chunk_message.choices > 0 then
        local first_choice = chunk_message.choices[1]
        chunk.content = first_choice.delta.content
        chunk.reasoning = first_choice.delta.reasoning
        chunk.finish_reason = first_choice.finish_reason

        if first_choice.delta.tool_calls and #first_choice.delta.tool_calls > 0 then
            local tool_call = first_choice.delta.tool_calls[1]

            chunk.tool_call = {
                name = tool_call["function"].name,
                id = tool_call.id,
                index = tool_call.index,
                arguments = tool_call["function"].arguments
            }
        end
    end

    return chunk
end

function OpenAIChatProvider:message(message_str)
    ---@type OpenAIMessageRespons
    local full_message = json.load(message_str)

    ---@type AssistantMessage
    local message = {
        role = "assistant",
        content = full_message.choices[1].message.content,
        reasoning = full_message.choices[1].message.reasoning,
        finish_reason = full_message.choices[1].message.finish_reason,
        usage = full_message.usage
    }

    return message
end

---@type IChunkReceiver
local ChunkReceiver = {}

function ChunkReceiver:receive(chunk)
    if chunk.reasoning then
        io.write(chunk.reasoning)
        io.flush()
    end

    if chunk.content then
        io.write(chunk.content)
        io.flush()
    end
end

-- local chat_client = ChatClient(OpenAIChatProvider)

-- local success, message = pcall(
--     chat_client.chat, chat_client,
--     {
--         {
--             role = "system",
--             content = "You are a helpful assistant"
--         },
--         {
--             role = "user",
--             content = "hello"
--         }
--     },
--     {
--         model = "qwen3.5",
--         base_url = "http://localhost:11434/v1",
--         api_key = "Ollama",
--         stream = true,
--         max_tokens = 1024000
--     },
--     nil, ChunkReceiver
-- )

-- if success then
--     print(message.content)
-- else
--     print(message)
-- end

-- print("\n\n------- assistant message -------\n\n")
-- print(message.role)
-- print(message.content)
-- print(message.reasoning)

-- local tool_executor = DefaultToolExecutor()

-- print(tool_executor:execute("get_weather", "id", "BeiJing"))

-- tool_executor:load("tests/tools")

-- print(tool_executor:execute("get_system_lang", "id", "BeiJing"))

-- print(TINY_VERSION)

-- local http_client = HttpClient()

-- local ret = http_client:send(
--     "post", "http://localhost:11434/v1/chat/completions",
--     {
--         stream = true,
--         model = "qwen3.5",
--         messages = {
--             {
--                 role = "system",
--                 content = "you are a helpful assistant"
--             },
--             {
--                 role = "user",
--                 content = "hello"
--             }
--         }
--     },
--     {
--         Authorization = "Bearer Ollama"
--     },
--     function (line)
--         if line ~= "data: [DONE]" then
--             local data_line = string.sub(line, 7)

--             local success, obj = pcall(json.load, data_line)

--             if success and obj.choices[1] then
--                 if obj.choices[1].delta.content then
--                     io.write(obj.choices[1].delta.content)
--                     io.flush()
--                 end
--             end
--         else
--             print("\n")
--         end
--     end
-- )

-- if type(ret) == "string" then
--     print(ret)
-- elseif type(ret) == "table" then
--     for _, l in ipairs(ret) do
--         print(l)
--     end
-- end

-- local ret = http_client:send("get", "http://localhost:11434/v1/models")

-- print(ret)

-- plugins will be loaded into current lua state, as it need to interactive with the agent
-- tiny.plugins.load("/path/to/load")

-- in the plugin.lua

-- local plugin = {
--     name = "myplugin", -- name of the plugin
--     description = "", -- description of the plugin
--     config = {}, -- default configuration, will be override with user specified
--     -- each plugin will have it optional configuration while register function called
--     register = function (ctx, conf)
--         -- add more chat client
--         ctx.chat_clients.add("openai", ChatClient(OpenAIChatProvider))
--         ctx.chat_clients.add("deepseek", ChatClient(DeepseekChatProvider))

--         -- we can write our own chat client with use exist chat client wrapper
--         ctx.chat_clients.add("ollama", OllamaChatClient)

--         -- prompt injections, used to update system prompt, only being called one time
--         ctx.prompt_updaters.add(MyPromptUpdater)

--         -- log system
--         ctx.logger.add(MyLogger)

--         -- loop lifecycle
--         ctx.loop_callbacks(MyCallbackHandlers)

--         --
--         ctx.user_input_handler(MyUserInputHandler)

--         --
--         ctx.model_selector(MyModelSelector)

--         -- try load tools under 'tools' folder under plugin folder
--         ctx.load_tools()
--     end
-- }

---@type AgentLoop
local agent_loop = require "tiny.agent.loop"

---@type AgentLoopContext
local ctx = {
    messages = {
        {
            role = "system",
            content = "you are a helpful assistant"
        },
        {
            role = "user",
            content = "What's the weather in Beijing?"
        }
    },

    options = {
        model = "qwen3.5",
        base_url = "http://localhost:11434/v1",
        api_key = "Ollama",
        stream = true,
        max_tokens = 1024000
    },

    tools = {
        {
            name = "get_weather",
            description = "get weather of specified city",
            parameters = {
                {
                    name = "city",
                    ["type"] = "string",
                    description = "city name",
                    required = true
                }
            }
        }
    }
}

---@type AgentLoopPlugins
local plugins = {
    chat_client = ChatClient(OpenAIChatProvider),
    chunk_receiver = ChunkReceiver,
    tool_executor = DefaultToolExecutor()
}

agent_loop(ctx, plugins)

-- print(ctx.messages[#ctx.messages].tool_calls[1])
