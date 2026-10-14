local SessionManager = {}

SessionManager.__index = SessionManager

function SessionManager.new()
    return setmetatable({}, SessionManager)
end

return SessionManager
