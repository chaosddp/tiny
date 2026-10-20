
---@class ISessionManager
local ISessionManager = {}

---@param session? string @session name, leave it empty to load default one
---@return Message[]
function ISessionManager:load_memory(session) end

--- switch to a new session, if give nil, then use default session
---@param session? string @session name to switch to
function ISessionManager:switch_session(session)end

---@param message UserMessage | AssistantMessage | ToolMessage
function ISessionManager:log_message(message) end

---@param name? string @name of session, leave it empty to use default name
---@return string @name of the session
---@return string @path to session
function ISessionManager:new_session(name) end