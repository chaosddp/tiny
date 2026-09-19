local function get_weather(o)
    return [[
        Condition: Cloudy
        Temperature: 17°C (feels comfortable/cool)
        Humidity: 85%
        Wind: East at 4 mph
        High Temperature: 23°C
        Low Temperature: 15°C
        Tonight: Temperatures will drop to around 15°C and 14°C later tonight.
    ]]
end

return {
    {
        name = "get_weather",
        func = get_weather,
        definition = {
            desc = "get weather of specified city", -- descript of the tool
            parameters = {
                city = {
                    ["type"] = "string",
                    desc = "city name",
                    required = true
                }
            }
        }
    }
}
