local platform = platform
local json = json

return function ()
    return json.dump(platform.info())
end
