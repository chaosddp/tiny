local ToolManager = require "default.tool_manager"

tiny.extensions.chat_clients["openai"] = require "default.openai"
tiny.extensions.chat_loop = require "default.chat_loop"
tiny.extensions.tool_manager = ToolManager.new()
