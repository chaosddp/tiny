local string = string
local json = json
local io = io
local pcall = pcall
local base64 = base64

---@class RunCommandOptions
---@field command string

---@param options string
local function run_powershell(options)
    local success, options = pcall(json.load, options)

    if not success then
        return options
    end

    ---@cast options RunCommandOptions
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

    -- force powershell output utf-8 charset
    local command = string.format(
        "$OutputEncoding = [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; %s",
        command
    )

    -- encode to utf-16 as powershell required
    local success, utf16_bytes = pcall(string.to_bytes, command, "utf-16")

    if success then
        local success, b64_command = pcall(base64.encode, utf16_bytes)

        if success then
            local command_to_run = string.format(
                "powershell -NonInteractive -encodedCommand %s",
                b64_command
            )
            local success, file = pcall(io.popen, command_to_run, "r")

            if success then
                local output = file:read("*a")
                file:close()

                return output
            end

            return file
        end

        return b64_command
    end

    return utf16_bytes
end

local definition = {
    name = "run_powershell",
    description = "run command in Microsoft Powershell on Windows system",
    parameters = {
        {
            name = "command",
            description = "the command need to run",
            required = true,
            type = "string"
        }
    }
}

return { definition = definition, func = run_powershell }
