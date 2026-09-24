local agent_loop = require "tiny.agent.loop"

---@class TinyAgent
local TinyAgent = {}

function TinyAgent:init(configs, extensions, tools)
    self.config = configs
    self.extensions = extensions
    self.tools = tools
    self.messages = {
        {
            role = "system",
            content = "you are a helpful assistant"
        }
    }
end

function TinyAgent:chat(model, message)
    local model_options = self.config.models[model]

    table.insert(self.messages, { role = "user", content = message })

    agent_loop({
        messages = self.messages,
        options = model_options,
        tools = self.tools
    },
        {
            chat_client = self.extensions.chat_clients[model_options.client],
            chunk_receiver = self.extensions.chunk_receiver,
            tool_executor = DefaultToolExecutor()
        })
end

return TinyAgent
