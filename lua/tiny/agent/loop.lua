local pcall = pcall
local table = table
local ipairs = ipairs

--- base agent loop for new user message
---@param ctx     AgentLoopContext
---@param plugins AgentLoopPlugins
local function agent_loop(ctx, plugins)
  local is_model_support_tools = (ctx.options.support_tools and plugins.tool_executor ~= nil)
    or false

  while true do
    local message = plugins.chat_client:chat(
      ctx.messages,
      ctx.options,
      is_model_support_tools and ctx.tools or nil,
      plugins.chunk_receiver
    )

    table.insert(ctx.messages, message)

    -- stop if we do not tool calls
    if not is_model_support_tools or message.finished_reason ~= "tool_calls"
      or message.tool_calls == nil or #message.tool_calls == 0 then
      break
    end

    ---@cast plugins.tool_executor - nil
    for _, tool_call in ipairs(message.tool_calls) do
      local tool_result = plugins.tool_executor:execute(
        tool_call.name,
        tool_call.id,
        tool_call.arguments
      )

      if plugins.chunk_receiver then
        plugins.chunk_receiver:receive({ tool_result = tool_result })
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
