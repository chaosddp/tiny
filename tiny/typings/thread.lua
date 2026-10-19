---@class Thread
local Thread = {}

--- if current thread is finished
---@return boolean
function Thread:is_finished() end

tiny.thread = {}

--- start a new thread with new lua state
---@param file_or_content string @lua script content or a file
---@return Thread
function tiny.thread.newThread(file_or_content, ...) end
