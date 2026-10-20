local ToolManager = require "default.tool_manager"
local SessionManager = require "default.session_manager"

tiny.extensions.chat_clients["openai"] = require "default.openai"
tiny.extensions.chat_loop = require "default.chat_loop"
tiny.extensions.tool_manager = ToolManager.new()
tiny.extensions.session_manager = SessionManager.new()
