local pcall = pcall
local table = table
local ipairs = ipairs

--- base agent loop for new user message
---@param ctx     AgentLoopContext
---@param extensions AgentLoopExtensions
local function agent_loop(ctx, extensions)
  local is_model_support_tools = true

  if is_model_support_tools and extensions.tool_executor == nil then
    is_model_support_tools = false
  end

  if is_model_support_tools and ctx.options.support_tools == false then
    is_model_support_tools = false
  end

  while true do
    local message = extensions.chat_client:chat(
      ctx.messages,
      ctx.options,
      is_model_support_tools and ctx.tools or nil,
      extensions.chunk_receiver
    )

    table.insert(ctx.messages, message)

    -- stop if we do not tool calls
    if not is_model_support_tools or message.finish_reason ~= "tool_calls"
      or message.tool_calls == nil or #message.tool_calls == 0 then
      break
    end

    ---@cast extensions.tool_executor - nil
    for _, tool_call in ipairs(message.tool_calls) do
      local tool_result = extensions.tool_executor:execute(
        tool_call.name,
        tool_call.id,
        tool_call.arguments
      )

      if extensions.chunk_receiver then
        extensions.chunk_receiver:receive({ tool_result = tool_result })
      end

      ---@type ToolMessage
      local tool_message = {
        role = "tool",
        content = tool_result,
        id = tool_call.id,
        name = tool_call.name
      }

      table.insert(ctx.messages, tool_message)
    end
  end
end

return agent_loop
