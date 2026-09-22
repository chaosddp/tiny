---@type ChatProvider
local OpenAIChatProvider = {}

function OpenAIChatProvider:request(messages, tools, options)
    local url = options.base_url .. "/chat/completions"

    local body = {
        model = options.model,
        stream = options.stream,
        max_tokens = options.max_tokens,
        messages = messages
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
    end

    return chunk
end

function OpenAIChatProvider:message(message_str)
    ---@type OpenAIMessageRespons
    local full_message = json.load(message_str)

    ---@type AssistantMessage
    local message = {
        role = "assistant",
        content = full_message.choices[1].message.content,
        reasoning = full_message.choices[1].message.reasoning,
        finished_reason = full_message.choices[1].message.finish_reason,
        usage = full_message.usage
    }

    return message
end

---@type IChunkReceiver
local ChunkReceiver = {}

function ChunkReceiver:receive(chunk)
    if chunk.reasoning then
        io.write(chunk.reasoning)
        io.flush()
    end

    if chunk.content then
        io.write(chunk.content)
        io.flush()
    end
end

local chat_client = ChatClient(OpenAIChatProvider)

local success, message = pcall(
    chat_client.chat, chat_client,
    {
        {
            role = "system",
            content = "You are a helpful assistant"
        },
        {
            role = "user",
            content = "hello"
        }
    },
    nil,
    {
        model = "qwen3.5",
        base_url = "http://localhost:11434/v1",
        api_key = "Ollama",
        stream = true,
        max_tokens = 1024000
    },
    ChunkReceiver
)

if success then
    print(message.content)
else
    print(message)
end

-- print("\n\n------- assistant message -------\n\n")
-- print(message.role)
-- print(message.content)
-- print(message.reasoning)
