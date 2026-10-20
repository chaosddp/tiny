---@alias LuaStatePrimaryValue boolean | integer | number | string

---@alias LuaStateValue LuaStatePrimaryValue | table<LuaStatePrimaryValue, LuaStatePrimaryValue>

---@class LuaState
local LuaState = {}

--- execute script or file
---@param script_or_file string @script content or file path to execute
function LuaState:execute(script_or_file) end

--- eval script
---@param script string @script content to eval
---@return LuaStateValue?
function LuaState:eval(script) end

lua = {}

---@return LuaState
function lua.newState() end
