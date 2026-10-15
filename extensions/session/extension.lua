local SessionManager = require "session.session"

local session_manager = SessionManager.new()

-- register event handlers, handlers will be called according to the register order one by one.
-- if any handler function return true, then will prevent event to pass to further handler
table.insert(tiny.event_handlers, session_manager)

-- register it as session manager
tiny.session_manager = session_manager
