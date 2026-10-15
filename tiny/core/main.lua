require "table_utils"

---@param extensions string[]
function tiny.load_extensions(extensions)
    for _, ext in ipairs(extensions) do
        require(ext .. ".extension")
    end
end
