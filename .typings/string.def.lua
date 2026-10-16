--- convert string into utf-8 byte string
---@param s  string
---@param b? "utf-8" | "utf-16"
---@return string
function string.to_bytes(s, b) end

--- convert os string  into utf8
---@param s string @os string
---@return string @utf8 string
function string.from_os_string(s) end
