local string = string

local function is_blank(s)
    return s == " " or s == "\n" or s == "\r" or s == "\t"
end

--- trim blanks from string
---@param s string
---@return string
function string.trim(s)
    local l = 1

    while is_blank(string.sub(s, l, l)) do
        l = l + 1
    end

    local r = #s
    while is_blank(string.sub(s, r, r)) do
        r = r - 1
    end

    if r <= l then
        return ""
    end

    return string.sub(s, l, r)
end
