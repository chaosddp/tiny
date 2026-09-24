---@param tool_call ToolCall
---@return table<string, any>
local function tool_call_to_openai(tool_call)
    return {
        id = tool_call.id,
        ["type"] = "function",
        ["function"] = {
            name = tool_call.name,
            arguments = tool_call.arguments
        }
    }
end

---@param message Message
---@return table<string, any>
local function message_to_openai(message)
    if message.role == "system" then
        ---@cast message SystemMessage
        return { role = "system", content = message.content }
    elseif message.role == "tool" then
        ---@cast message ToolMessage
        return { role = "tool", content = message.content, name = message.name }
    elseif message.role == "assistant" then
        ---@cast message AssistantMessage
        local openai_tool_calls = nil

        if message.tool_calls then
            openai_tool_calls = table.map(message.tool_calls, function (t)
                return tool_call_to_openai(t)
            end)
        end

        return {
            role = "assistant",
            content = message.content,
            reasoning_content = message.reasoning,
            reasoning_details = message.reasoning_details,
            tool_calls = openai_tool_calls
        }
    else
        local m = { role = "user" }

        if message.content then
            ---@cast message TextUserMessage
            m.content = message.content
        else
            m.parts = {}

            ---@cast message PartsUserMessage
            for _, part in ipairs(message.parts) do
                if part.type == "text" then
                    table.insert(m.parts, { ["type"] = "text", text = part.text })
                elseif part.type == "file" then
                    table.insert(m.parts, { ["type"] = "file_url", file_url = { url = part.file } })
                elseif part.type == "video" then
                    table.insert(m.parts, {
                        ["type"] = "video_url",
                        video_url = { url = part.video }
                    })
                else
                    if part.image and part.image then
                        local detail = part.image_detail or "auto"
                        table.insert(m.parts, {
                            ["type"] = "image_url",
                            image_url = { url = part.image, detail = detail }
                        })
                    end
                end
            end
        end

        return m
    end
end

---@param tool Tool
---@return table<string, any>
local function tool_to_openai(tool)
    local openai_tool_parameters = {
        type = "object",
        required = nil,
        parameters = nil
    }

    if tool.parameters and #tool.parameters > 0 then
        openai_tool_parameters.required = {}
        openai_tool_parameters.properties = {}

        for _, p in ipairs(tool.parameters) do
            if p.required then
                table.insert(openai_tool_parameters.required, p.name)
            end

            openai_tool_parameters.properties[p.name] = {
                type = p.type,
                description = p.description
            }
        end
    end

    return {
        type = "function",
        ["function"] = {
            name = tool.name,
            description = tool.description,
            parameters = openai_tool_parameters
        }
    }
end

---@class OpenAIChatProvider: ChatProvider
local OpenAIChatProvider = {}

function OpenAIChatProvider:request(messages, options, tools)
    local url = options.base_url .. "/chat/completions"

    local openai_tools = nil

    if tools and #tools > 0 then
        openai_tools = table.map(tools, function (t) return tool_to_openai(t) end)
    end

    local body = {
        model = options.model,
        messages = table.map(messages, function (m) return message_to_openai(m) end),
        tools = openai_tools,
        stream = options.stream or true,
        max_tokens = options.max_tokens or 8000
    }

    ---@type RequestOptions
    local request_options = {
        url = url,
        body = json.dump(body),
        headers = {
            Authorization = "Bearer " .. options.api_key,
            ["Content-Type"] = "application/json"
        }
    }

    return true, request_options
end

function OpenAIChatProvider:chunk(chunk_str)
    local chunk_message = json.load(chunk_str)

    ---@type Chunk
    local chunk = {}

    if chunk_message.choices and #chunk_message.choices > 0 then
        local first_choice = chunk_message.choices[1]
        chunk.content = first_choice.delta.content
        chunk.reasoning = first_choice.delta.reasoning
        chunk.finish_reason = first_choice.finish_reason

        if first_choice.delta.tool_calls and #first_choice.delta.tool_calls > 0 then
            local tool_call = first_choice.delta.tool_calls[1]

            chunk.tool_call = {
                name = tool_call["function"].name,
                id = tool_call.id,
                index = tool_call.index,
                arguments = tool_call["function"].arguments
            }
        end
    end

    return chunk
end

function OpenAIChatProvider:message(message_str)
    ---@type OpenAIMessageRespons
    local full_message = json.load(message_str)

    if full_message and full_message.choices and #full_message.choices > 0 then
        ---@type AssistantMessage
        local message = {
            role = "assistant",
            content = full_message.choices[1].message.content,
            reasoning = full_message.choices[1].message.reasoning,
            finish_reason = full_message.choices[1].message.finish_reason,
            usage = full_message.usage
        }

        return message
    end

    return nil
end

return OpenAIChatProvider
