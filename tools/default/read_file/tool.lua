local pcall = pcall
local json = json
local string = string
local path = path
local table = table

---@param options string
---@return string
local function read_file(options)
    local success, options = pcall(json.load, options)

    if not success then
        return options
    end

    ---@type string?
    local file = options.file

    if file == nil then
        return "parameter file is not provided"
    end

    file = string.trim(file)

    if #file == 0 then
        return "file is empty or blank"
    end

    if not path.exists(file) or not path.is_file(file) then
        return "invalid file, not exist or is not a file"
    end

    local success, file = pcall(io.open, file, "r")

    if not success then
        return "fail to open the file: " .. file
    end

    if file == nil then
        return "fail to open file, unknow error"
    end

    local limit = options.limit

    local content = ""

    if limit == nil then
        content = file:read("*a")
    else
        local lines = {}
        local n = 0

        for line in file:lines() do
            table.insert(lines, line)

            n = n + #line
        end

        -- NOTE: this is byte length, not utf-8, may cause invalid utf-8 char
        content = table.concat(lines, "\n").sub(0, limit)
    end

    return content
end

return read_file
