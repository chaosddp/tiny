---@param messages Message[]
---@param options  ChatOptions
local function base_chat_loop(messages, options)
    local tools = nil
    if tiny.extensions.tool_executor then
        tools = tiny.extensions.tool_executor:tools()
    end

    local client_name = options.client or "openai"

    ---@type ChatClient?
    local chat_client = tiny.extensions.chat_clients[client_name]

    if chat_client == nil then
        error("chat client not exist: " .. client_name)
    end

    while true do
        local success, assistant_message = chat_client:chat(
            messages,
            options,
            tools,
            tiny.chunk -- this can be nil
        )

        ---
        if success and assistant_message then
            ---@diagnostic disable-next-line: unnecessary-if
            ---@cast assistant_message AssistantMessage
            if tiny.message then
                tiny.message(assistant_message)
            end

            table.insert(messages, assistant_message)

            if assistant_message.finish_reason ~= "tool_calls"
                or assistant_message.tool_calls == nil
                or #assistant_message == 0
                or tiny.extensions.tool_executor == nil then
                break
            end

            for _, tool_call in ipairs(assistant_message.tool_calls) do
                -- TODO: tool call with tiny.extensions.tool_executor
                local result = tiny.extensions.tool_executor:call(
                    tool_call.name,
                    tool_call.arguments
                )

                ---@type ToolMessage
                local tool_message = {
                    role = "tool",
                    content = result,
                    name = tool_call.name,
                    id = tool_call.id
                }

                ---@diagnostic disable-next-line: unnecessary-if
                if tiny.message then
                    tiny.message(tool_message)
                end

                table.insert(messages, tool_message)
            end
        else
            -- TODO: error handling
            break
        end
    end
end

return base_chat_loop
