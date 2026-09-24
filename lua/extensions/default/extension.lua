local ChunkReceiver = require "chunk_receiver"

local function register(ctx, configs)
    ctx.chunk_receiver.set(ChunkReceiver())
end

return register
