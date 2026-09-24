local OpenAIChatProvider = require "extensions.openai.openai_provider"

---@param ctx      ExtensionRegisterContext
---@param configs? string
local function register(ctx, configs)
    ctx.add_chat_client("openai", ChatClient(OpenAIChatProvider))
end

return register
