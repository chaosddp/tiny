require 'table_utils'
local OpenAIChatProvider = require "openai"

tiny.chat_clients.openai = OpenAIChatProvider.new()
