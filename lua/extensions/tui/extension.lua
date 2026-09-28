
-- we create a wrapper here, as current LuaChatClient expect a luatable as chunk receiver

local Wrapper = {inner=DefaultWrapper}

function Wrapper:chunk(chunk)
    return self.inner:chunk(chunk)
end

function Wrapper:message(message)
    self.inner:message(message)
end

function Wrapper:show_reasoning(b)
    self.inner:show_reasoning(b)
end

function Wrapper:input()
    return self.inner:input()
end

---@param ctx ExtensionRegisterContext
local function register(ctx, configs)
    ctx.set_chunk_recever(Wrapper)
    ctx.set_input_source(Wrapper)
end

return register
