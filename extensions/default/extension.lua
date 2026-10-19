local OpenAIChatClient = require "default.openai"

local base_chat_loop = require "default.chat_loop"

tiny.extensions.chat_clients["openai"] = OpenAIChatClient
tiny.extensions.chat_loop = base_chat_loop
