local json = json
local pcall = pcall
local string = string
local table = table

local function write_file(options)
    local success, options = pcall(json.load, options)

    if not success then
        return "Fail to parse the parameter, require a valid json string"
    end

    local file = options.file
    local content = options.content

    if file == nil then
        return "file is not provided"
    end

    file = string.trim(file)

    if not path.exists(file) or not path.is_file(file) then
        return "file not exist, or not a file"
    end

    if content == nil then
        return "content is not provided"
    end

    local success, fp = pcall(io.open, file, "w")

    if not success then
        return "fail to open file: " .. fp
    end

    if fp == nil then
        return "fail to open file with unknown error"
    end

    fp:write(content)
    fp:flush()
    fp:close()

    return "success to write"
end

local definition = {
    name = "write_file",
    description = "Write content to file, it will over write the file with new content",
    parameters = {
        {
            name = "file",
            description = "file to write to",
            required = true,
            type = "string"
        },
        {
            name = "content",
            description = "text content to write to",
            required = true,
            ["type"] = "string"
        }
    }
}

return { definition = definition, func = write_file }
