local tiny = {}

-- called before started, pass current configurations
function tiny.conf(...) end

--- called after 'tiny.conf' to do initialization
function tiny.load(...) end

--- handle 'session' command
function tiny.session(...) end

--- handle 'lua' command to execute specified files
function tiny.file(...) end

--- handle default chat command
function tiny.chat(...) end

--- handle chunks if enbled stream mode
function tiny.chunk(...) end

--- handle message from server
function tiny.message(...) end

--- handle error that cause application crash
function tiny.error(...) end

return tiny
