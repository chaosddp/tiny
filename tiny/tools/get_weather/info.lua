return {
    name = "get_weather",
    description = "get weather of specified city",
    parameters = {
        {
            name = "city",
            ["type"] = "string",
            desc = "city name",
            required = true
        }
    }
}
