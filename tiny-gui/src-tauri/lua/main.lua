require 'table_utils'
local OpenAIChatProvider = require "openai"

-- tiny.default_model = "ollama/qwen3.5"

tiny.chat_clients.openai = OpenAIChatProvider.new()

-- tiny.configs.models = {
--     ["ollama/qwen3.5"] = {
--         name = "qwen3.5",
--         base_url = "http://localhost:11434/v1",
--         api_key = "ollama",
--         max_tokens = 64000,
--         features = {
--             vision = false,
--             thinking = true,
--             decision = false
--         }
--     }
-- }
