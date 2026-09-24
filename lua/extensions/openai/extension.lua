local OpenAIChatProvider = require "openai_provider"

local function register(ctx, configs)
    ctx.chat_clients.add("openai", ChatClient(OpenAIChatProvider))
end

return {
    name = "builtin_openai_chat_client",
    description = "Builtin OpenAI chat client",
    register = register
}
