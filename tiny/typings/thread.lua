---@class Thread
local Thread = {}

--- if current thread is finished
---@return boolean
function Thread:is_finished() end

tiny.thread = {}

--- start a new thread with new lua state
--- local thread = tiny.thread.newThread(
---      [[
---
---  local n1, n2 = arg[1] or 1, arg[2] or 10
---
---  local channel = tiny.channel.get("chat_channel")
---
---  for i=n1, n2 do
---      channel:send(i)
---  end
---
---  ]]
---  )
---
---  local channel = tiny.channel.get("chat_channel")
---
---  while not thread:is_finished() do
---
---      local v = channel:recv("j")
---
---      if v ~= nil then
---
---          print(v)
---
---      end
---
---  end
---@param file_or_content string @lua script content or a file
---@return Thread
function tiny.thread.newThread(file_or_content, ...) end
