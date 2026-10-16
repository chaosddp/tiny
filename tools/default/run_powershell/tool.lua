local string = string

---@class RunCommandOptions
---@field command string

---@param options string
local function run_powershell(options)
    ---@type RunCommandOptions
    local options = json.load(options)

    local command = options.command

    if command == nil then
        return "run_powershell command must provide 'command' to run"
    end

    command = string.trim(command)

    -- make sure the command if valid
    if #command == 0 then
        return "invalid command to run"
    end

    local start_cmd = string.sub(command, 1, string.find(command, " "))

    if string.find(start_cmd, "sudo") == 1 then
        return "sudo is not allowed to run."
    end

    if string.find(start_cmd, "Invoke-Expression") == 1 or string.find(start_cmd, "iex") == 1 then
        return "iex or Invoke-Expression is not allowed to run"
    end

    -- encode as utf-16
    local encoded_command = base64.encode(string.to_bytes(command, "utf-16"))

    local file = io.popen(string.format("powershell -encodedCommand %s", encoded_command), "r")

    local output = file:read("*a")

    local success, detail = pcall(string.from_os_string, output)

    return detail
end

return run_powershell
