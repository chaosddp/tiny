local path = path
local table = table
local tostring = tostring
local ipairs = ipairs
local json = json
local string = string
local io = io
local tiny_folder = ".tiny"
local sessions_json_file = string.format("%s/%s", tiny_folder, "sessions.json")

---@class SessionManager
---@field private sessions           any[]
---@field private last_session_name? string
---@field private id                 int
---@field private memory_file?       file
local SessionManager = {}
SessionManager.__index = SessionManager

function SessionManager.new()
    if not path.exists(tiny_folder) then
        path.mkdir(tiny_folder)
    end

    if not path.exists(sessions_json_file) then
        local default_sessions_str = json.dump({ sessions = {}, last = nil, id = 0 }, true)

        local file = io.open(sessions_json_file, "w")

        if file then
            file:write(default_sessions_str)
            file:flush()
            file:close()
        end
    end

    local sessions = {}
    local last_session = nil
    local id = 0

    local file = io.open(sessions_json_file)

    if file then
        local sessions_str = file:read("*a")

        file:close()

        local sessions_json = json.load(sessions_str)

        sessions = sessions_json.sessions
        last_session = sessions_json.last
        id = sessions_json.id or 0
    end

    return setmetatable({
        sessions = sessions,
        last_session_name = last_session,
        id = id,
        memory_file = nil
    }, SessionManager)
end

---@param session string
---@return Message[]
function SessionManager:load_memory(session)
    local session_folder = string.format("%s/%s", tiny_folder, session)
    local memory_file = string.format("%s/%s", session_folder, "memory.json")

    local file = io.open(memory_file)

    if file then
        local messages = {}

        for line in file:lines() do
            ---@cast line string
            table.insert(messages, json.load(line))
        end

        return messages
    end

    return {}
end

---@return string
function SessionManager:new_session()
    self.id = self.id + 1
    local session_name = "session_" .. tostring(self.id)

    table.insert(self.sessions, session_name)

    local session_folder = string.format("%s/%s", tiny_folder, session_name)

    path.mkdir(session_folder)

    self:save_state()

    return session_name
end

---@param session string
function SessionManager:switch_session(session)
    -- check if the session exist

    for _, src_session in ipairs(self.sessions) do
        if session == src_session then
            local session_folder = string.format("%s/%s", tiny_folder, session)

            if path.exists(session_folder) then
                self.last_session_name = session

                if self.memory_file ~= nil then
                    self.memory_file:close()
                end

                local memory_file = string.format("%s/%s", session_folder, "memory.json")

                self.memory_file = io.open(memory_file, "a+")

                self:save_state()
            end
        end
    end
end

---@return string?
function SessionManager:last_session()
    return self.last_session_name
end

function SessionManager:save_state()
    local file = io.open(sessions_json_file, "w+")

    local sessions = {
        sessions = self.sessions,
        last = self.last_session_name,
        id = self.id
    }

    if file then
        file:write(json.dump(sessions, true))

        file:close()
    end
end

---@param message Message
function SessionManager:message(message)
    -- here we receive messages

    if self.memory_file then
        local message_json_str = json.dump(message)

        self.memory_file:write(message_json_str)
        self.memory_file:write("\n")
        self.memory_file:flush()
    end
end

---@param session string
---@return string
function SessionManager:memory_to_md(session)
    local result = {}

    local session_folder = string.format("%s/%s", tiny_folder, session)
    local memory_file = string.format("%s/%s", session_folder, "memory.json")

    local file = io.open(memory_file)

    if file then
        for line in file:lines() do
            ---@cast line string
            local message = json.load(line)

            table.insert(result, string.format("## %s\n", message.role))

            if message.role == "system" then
                table.insert(result, string.format("\n%s\n\n", message.content))
            elseif message.role == "tool" then
                --- meta
                table.insert(
                    result,
                    string.format("---\nid: %s\nname: %s\n---\n\n", message.id, message.name)
                )

                -- content
                table.insert(result, string.format("%s\n\n", message.content))
            elseif message.role == "assistant" then
                -- reasoning
                if message.reasoning and #message.reasoning > 0 then
                    table.insert(
                        result,
                        string.format("### reasoning\n\n%s\n\n", message.reasoning)
                    )
                end

                if message.content and #message.content > 0 then
                    table.insert(result, string.format("### content\n\n%s\n\n", message.content))
                end

                if message.tool_calls and #message.tool_calls > 0 then
                    table.insert(
                        result,
                        string.format(
                            "### tool calls\n\n```json\n%s\n```\n\n",
                            json.dump({
                                tool_calls = message.tool_calls
                            }, true)
                        )
                    )
                end
            else
                if message.parts and #message.parts > 0 then
                    for _, part in ipairs(message.parts) do
                        if part.type == "text" then
                            table.insert(result, string.format("### text\n\n%s\n\n", part.text))
                        elseif part.type == "file" then
                            table.insert(result, string.format("### file\n\n%s\n\n", part.file))
                        elseif part.type == "video" then
                            table.insert(result, string.format("### video\n\n%s\n\n", part.video))
                        else
                            table.insert(result, string.format("### image\n\n%s\n\n", part.image))
                        end
                    end
                elseif message.content then
                    table.insert(result, string.format("%s\n\n", message.content))
                end
            end
        end
    end

    return table.concat(result)
end

return SessionManager
