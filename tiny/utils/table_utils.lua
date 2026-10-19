local table = table
local ipairs = ipairs

---@generic T
---@param arr  T[]
---@param func fun(T): boolean
---@return T[]
function table.filter(arr, func)
    local result = {}

    for _, m in ipairs(arr) do
        if func(m) then
            table.insert(m)
        end
    end

    return result
end

---@generic T
---@generic O
---@param arr  T[]
---@param func fun(m: T): O
---@return O[]
function table.map(arr, func)
    local result = {}

    for _, m in ipairs(arr) do
        table.insert(result, func(m))
    end

    return result
end
