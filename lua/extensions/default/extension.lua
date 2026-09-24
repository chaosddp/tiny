local ChunkReceiver = require "extensions.default.chunk_receiver"
local DefaultInputSource = require "extensions.default.input_source"

---@param ctx ExtensionRegisterContext
local function register(ctx, configs)
    ctx.set_chunk_recever(ChunkReceiver())
    ctx.set_input_source(DefaultInputSource)
end

return register
