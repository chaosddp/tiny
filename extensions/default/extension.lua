local SessionManager = require "default.session_manager"
local OpenAIChatBackend = require "default.openai"

tiny.backends:tool_executor({})
tiny.backends:event_hub({})
tiny.backends:session_manager(SessionManager.new())

tiny.backends:chat_client("openai", OpenAIChatBackend.new())