return function (options)
    options = json.load(options)
    local city = options.city

    return "Sunny. Highs in the lower 60s. Monday Night. Partly cloudy. Lows in the mid 40s in " .. city
end
