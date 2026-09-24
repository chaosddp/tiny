local path = path
local pcall = pcall
local pairs = pairs
local ipairs = ipairs
local json = json
local string = string
local table = table
local os = os
local io = io

local ExtensionManager = require "tiny.agent.extension_manager"
local TinyAgent = require "tiny.agent.agent"

function load_tool_definitions()
    local tool_definitions = {}
    local source_base = path.source_base_dir()
    local glob_pattern = source_base .. "/lua/tools/*/tool.json"

    local tool_files = path.glob(glob_pattern)

    for _, tool_file in ipairs(tool_files) do
        local success, file = pcall(io.open, tool_file)

        if success and file then
            local tool_def_str = file:read("*a")

            table.insert(tool_definitions, json.load(tool_def_str))
        else
            print("fail to load tool definition: " .. file)
        end
    end

    return tool_definitions
end

function run(configs)
    -- update api_key for models
    if configs.models then
        for k, v in pairs(configs.models) do
            if k and #k > 0 then
                ---@type string
                local api_key = v["api_key"]

                if string.sub(api_key, 1, 4) == "env:" then
                    local env_var_key = string.sub(api_key, 5)

                    local key_from_env = os.getenv(env_var_key)

                    if key_from_env == nil then
                        error("environment variable: " .. env_var_key .. " not exist")
                    end

                    configs.models[k]["api_key"] = key_from_env
                end
            end
        end
    end

    ---@type Extensions
    local extensions = ExtensionManager:load(configs, configs.extensions)

    local tools = load_tool_definitions()

    if extensions.chunk_receiver then
        extensions.chunk_receiver:show_reasoning(configs.show_reasoning and true or false)
    end

    local input_source = extensions.input_source

    if input_source then
        TinyAgent:init(configs, extensions, tools)

        while true do
            local user_message = input_source:input()

            TinyAgent:chat(configs.default_model, user_message)
        end
    else
        print("input source is nil")
    end
end

return run
