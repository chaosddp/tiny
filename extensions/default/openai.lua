local table = table
local string = string
local pairs = pairs
local ipairs = ipairs
local json = json
local pcall = pcall

--- NOTE: definition is not completed
---@class OpenAIMessage
---@field role          string
---@field content       string
---@field reasoning?    string
---@field finish_reason string

---@class OpenAIChoice
---@field index   integer
---@field message OpenAIMessage

---@class OpenAIMessageRespons
---@field id                 string
---@field object             string
---@field created            integer
---@field model              string
---@field system_fingerprint string
---@field choices            OpenAIChoice[]
---@field usage              Usage

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

---@class OpenAIChatClient: ChatClient
---@field private client LuaHttpClient
local OpenAIChatClient = {
    client = HttpClient()
}

function OpenAIChatClient:chat(messages, options, tools, chunk_callback)
    local req_options = self:make_request_options(messages, options, tools)

    if options.stream then
        ---@type AssistantMessage
        local message = { role = "assistant", finish_reason = "stop" }

        local content_table = {}
        local reasoning_table = {}

        function _process_chunk(line_str)
            local chunk = self:process_chunk(line_str)

            if chunk then
                if chunk_callback then
                    chunk_callback(chunk)
                end

                if chunk.content then
                    table.insert(content_table, chunk.content)
                end

                if chunk.reasoning then
                    table.insert(reasoning_table, chunk.reasoning)
                end

                if chunk.finish_reason then
                    message.finish_reason = chunk.finish_reason
                end

                if chunk.tool_call then
                    if message.tool_calls == nil then
                        message.tool_calls = {}
                    end

                    table.insert(message.tool_calls, chunk.tool_call)
                end
            end
        end

        local pcall_success, request_success, resp = pcall(
            self.client.send,
            self.client,
            "post",
            req_options.url,
            req_options.body,
            req_options.headers,
            _process_chunk
        )

        if not pcall_success then
            return false, { status_code = nil, message = request_success }
        end

        if not request_success then
            ---@cast resp HttpError
            return false, resp
        end

        message.content = #content_table > 0 and table.concat(content_table) or nil
        message.reasoning = #reasoning_table > 0 and table.concat(reasoning_table) or nil

        return true, message
    else
        local pcall_success, request_success, resp = pcall(
            self.client.send,
            self.client,
            "post",
            req_options.url,
            req_options.body,
            req_options.headers,
            nil
        )

        if not pcall_success then
            return false, { status_code = nil, message = request_success }
        end

        if not request_success then
            ---@cast resp HttpError
            return false, resp
        end

        ---@cast resp string
        return true, self:process_message(resp)
    end
end

---@param messages Message[]
---@param options  ChatOptions
---@param tools?   Tool[]
---@return RequestOptions @request options used to call HttpClient.send
function OpenAIChatClient:make_request_options(messages, options, tools)
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

    return request_options
end

---@param chunk_str string
---@return Chunk?
function OpenAIChatClient:process_chunk(chunk_str)
    if chunk_str and #chunk_str > 7 and chunk_str:startswith("data: ") then
        local data_str = chunk_str:sub(7)

        if data_str == "[DONE]" then return { done = true} end

        local chunk_message = json.load(data_str)

        ---@type Chunk
        local chunk = { id = chunk_message.id }

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

    return nil
end

function OpenAIChatClient:process_message(message_str)
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

return OpenAIChatClient
