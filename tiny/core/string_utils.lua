local string = string

--- trim blanks from string
---@param s string
---@return string
function string.trim(s)
    local l = 1
    while string.sub(s, l, l) == " " do
        l = l + 1
    end

    local r = #s
    while string.sub(s, r, r) == " " do
        r = r - 1
    end

    if r <= l then 
        return ""
    end

    return string.sub(s, l, r)
end
