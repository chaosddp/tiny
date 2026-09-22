

--- application will call this function to do plugin registetration
local function register(ctx)

end

--- 
return {
    name = "openai",
    description = "provide openai compatible chat interface",
    register = register
}