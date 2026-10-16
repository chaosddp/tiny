---@param file string
---@return string
function memory_to_md(memory_file)
    local result = {}

    local file = io.open(memory_file)

    if file then
        for line in file:lines() do
            ---@cast line string
            local message = json.load(line)

            table.insert(result, string.format("## %s\n", message.role))

            if message.role == "system" then
                table.insert(result, string.format("\n%s\n\n", message.content))
            elseif message.role == "tool" then
                --- meta
                table.insert(
                    result,
                    string.format("---\nid: %s\nname: %s\n---\n\n", message.id, message.name)
                )

                -- content
                table.insert(result, string.format("%s\n\n", message.content))
            elseif message.role == "assistant" then
                -- reasoning
                if message.reasoning and #message.reasoning > 0 then
                    table.insert(
                        result,
                        string.format("### reasoning\n\n%s\n\n", message.reasoning)
                    )
                end

                if message.content and #message.content > 0 then
                    table.insert(result, string.format("### content\n\n%s\n\n", message.content))
                end

                if message.tool_calls and #message.tool_calls > 0 then
                    table.insert(
                        result,
                        string.format(
                            "### tool calls\n\n```json\n%s\n```\n\n",
                            json.dump({
                                tool_calls = message.tool_calls
                            }, true)
                        )
                    )
                end
            else
                if message.parts and #message.parts > 0 then
                    for _, part in ipairs(message.parts) do
                        if part.type == "text" then
                            table.insert(result, string.format("### text\n\n%s\n\n", part.text))
                        elseif part.type == "file" then
                            table.insert(result, string.format("### file\n\n%s\n\n", part.file))
                        elseif part.type == "video" then
                            table.insert(result, string.format("### video\n\n%s\n\n", part.video))
                        else
                            table.insert(result, string.format("### image\n\n%s\n\n", part.image))
                        end
                    end
                elseif message.content then
                    table.insert(result, string.format("%s\n\n", message.content))
                end
            end
        end
    end

    return table.concat(result)
end

if #arg == 2 then
    local memory_file = arg[1]
    local output_file = arg[2]

    assert(memory_file ~= nil, "memory file cannot be empty")
    assert(output_file ~= nil, "output file cannot be empty")

    local content = memory_to_md(memory_file)

    local file = io.open(output_file, "w+")

    if file then
        file:write(content)
        file:flush()
        file:close()
    end
end
