local working_dir_root = string.format("%s/.tiny", path.working_dir())
local sessions_file = string.format("%s/session.json", working_dir_root)

print(sessions_file)

---@class SessionManager: ISessionManager
---@field private cur_memory_file? file
---@field private session_settings table<string, any>
local SessionManager = {}

SessionManager.__index = SessionManager

---@return SessionManager
function SessionManager.new()
    local session_settings = {
        default = nil,
        id = 0,
        sessions = {},
        manager = "default.session_manager"
    }

    if not path.exists(sessions_file) then
        local settings_json_str = json.dump(session_settings, true)

        local file = io.open(sessions_file, "w")

        if file then
            file:write(settings_json_str)
            file:flush()
            file:close()
        else
            error("fail to write sessions.json")
        end
    end

    local file = io.open(sessions_file)

    if file then
        local settings_json_str = file:read("*a")
        session_settings = json.load(settings_json_str)
    else
        error("fail to open sessions.json")
    end

    return setmetatable({
        ---@type file?
        cur_memory_file = nil,
        session_settings = session_settings
    }, SessionManager)
end

function SessionManager:load_memory(name)
    name = name or self.session_settings.default

    if name == nil then
        error("call switch_session first to setup a default session")
    end

    local _, session_path = self:new_session(name)

    local memory_file = string.format("%s/memory.jsonl", session_path)

    local fp = io.open(memory_file)

    if fp then
        local messages = {}

        for line in fp:lines() do
            ---@cast line string
            if line and #line > 0 then
                table.insert(messages, json.load(line))
            end
        end

        return messages
    end

    return {}
end

function SessionManager:new_session(name)
    self.session_settings.id = self.session_settings.id + 1

    name = name or string.format("session-%d", self.session_settings.id)

    -- try to find the session
    local session_exist = false
    for _, s in ipairs(self.session_settings.sessions) do
        if s == name then
            session_exist = true
        end
    end

    local session_path = string.format("%s/%s", working_dir_root, name)

    if not path.exists(session_path) then
        path.mkdir(session_path)
    end

    if not session_exist then
        table.insert(self.session_settings.sessions, name)
    end

    return name, session_path
end

---@param session? string
function SessionManager:switch_session(session)
    local session_path = nil

    if session == nil then
        if self.session_settings.default == nil then
            -- new one, we need to create a default session
            session, session_path = self:new_session()
        else
            session, session_path = self:new_session(self.session_settings.default)
        end
    else
        _, session_path = self:new_session(session)
    end

    local memory_file = string.format("%s/memory.jsonl", session_path)

    if self.cur_memory_file ~= nil then
        self.cur_memory_file:flush()
        self.cur_memory_file:close()

        self.cur_memory_file = nil
    end

    self.cur_memory_file = io.open(memory_file, "a+")

    if self.cur_memory_file == nil then
        error("fail to create memory file")
    end

    self.session_settings.default = session

    self:save()
end

function SessionManager:log_message(message)
    if self.cur_memory_file then
        self.cur_memory_file:write(string.format("%s\n", json.dump(message)))
        self.cur_memory_file:flush()
    end
end

function SessionManager.__gc(t)
    if t.cur_memory_file then
        t.cur_memory_file:flush()
        t.cur_memory_file:close()

        t.cur_memory_file = nil
    end
end

function SessionManager:save()
    local settings_json_str = json.dump(self.session_settings)

    local fp = io.open(sessions_file, "w")

    if fp then
        fp:write(settings_json_str)
        fp:flush()
        fp:close()
    end
end

return SessionManager
