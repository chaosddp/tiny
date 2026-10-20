local platform = platform
local json = json

local function get_platform()
    return json.dump(platform.info())
end

local definition = {
    name = "get_platform",
    description = "get current platform system information",
    parameters = {}
}

return { definition = definition, func = get_platform }
