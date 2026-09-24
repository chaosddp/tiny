local table = table
local json = json
local pairs = pairs
local ipairs = ipairs

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

-- ---@type AgentLoop
-- local agent_loop = require "tiny.agent.loop"

-- ---@type AgentLoopContext
-- local ctx = {
--     messages = {
--         {
--             role = "system",
--             content = "you are a helpful assistant"
--         },
--         {
--             role = "user",
--             content = "What's the weather in Beijing?"
--         }
--     },

--     options = {
--         model = "qwen3.5",
--         base_url = "http://localhost:11434/v1",
--         api_key = "Ollama",
--         stream = true,
--         max_tokens = 1024000
--     },

--     tools = {
--         {
--             name = "get_weather",
--             description = "get weather of specified city",
--             parameters = {
--                 {
--                     name = "city",
--                     ["type"] = "string",
--                     description = "city name",
--                     required = true
--                 }
--             }
--         }
--     }
-- }

-- ---@type AgentLoopExtensions
-- local extensions = {
--     chat_client = ChatClient(OpenAIChatProvider),
--     chunk_receiver = ChunkReceiver,
--     tool_executor = DefaultToolExecutor()
-- }

-- agent_loop(ctx, extensions)

-- print(ctx.messages[#ctx.messages].tool_calls[1])

local t = {
    -- configs = {
    --     -- configuration for builtin openai chat client
    --     builtin_openai_chat_client = {
    --         a = 1
    --     }
    -- }
}

local extension_manager = ExtensionManager()

extension_manager:load("D:/projects/rust/tiny-agent/lua/extensions/openai")

print(extension_manager)
print(package.path)
