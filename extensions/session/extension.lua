local myobj = {}

function myobj:message(message)
    -- print(message)
end

-- register event handlers, handlers will be called according to the register order one by one.
-- if any handler function return true, then will prevent event to pass to further handler
table.insert(tiny.event_handlers, myobj)

-- register a memory loader function that used to load memory from specified session
-- tiny.memory_loader = function (session_name)
-- end

