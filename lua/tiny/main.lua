local ExtensionManager = require "tiny.agent.extension_manager"
local TinyAgent = require "tiny.agent.agent"

function run()
    local configs = {
        models = {
            ["ollama/qwen3.5"] = {
                model = "qwen3.5",
                provider = "ollama",
                client = "openai",
                base_url = "http://localhost:11434/v1",
                api_key = "Ollama",
                stream = true,
                max_tokens = 1024000
            }
        },
        extensions = {
            "openai",
            "default"
        }
    }

    local extensions = ExtensionManager:load(configs, configs.extensions)

    -- TODO: load tool definitions

    local input_source = extensions.input_source

    if input_source then
        TinyAgent:init(configs, extensions)

        while true do
            local user_message = input_source:input()

            TinyAgent:chat("ollama/qwen3.5", user_message)
        end
    else
        print("input source is nil")
    end
end

return run
