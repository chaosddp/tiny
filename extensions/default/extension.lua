local OpenAIChatProvider = require "default.openai"
local SessionManager = require "default.session"
local prompts = require "default.prompt"


local session_manager = SessionManager.new()

-- register session manager as an event handler to collect messages
table.insert(tiny.event_handlers, session_manager)

-- update prompt
table.insert(tiny.event_handlers, prompts)

-- register it as session manager
tiny.session_manager = session_manager

-- openai style provider
tiny.chat_clients.openai = OpenAIChatProvider.new()
