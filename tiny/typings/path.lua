---@class PathInfo
---@field is_dir    boolean @if the path is a dir
---@field is_file   boolean @if the path is a file
---@field exists    boolean @if the path exists
---@field extension string? @extension of the path if it is a file
---@field file_name string? @file name (with extension) of the path if it is a file
---@field base_name string? @file base name (without extension) of the path if it is a file
---@field parent    string? @parent of the path
local PathInfo = {}

path = {}

--- check if the path is a file
---@param s string @path to check
---@return boolean
function path.is_file(s) end

--- check if the path is a dir
---@param s string @path to check
---@return boolean
function path.is_dir(s) end

--- check if path exist
---@param s string @path to check
---@return boolean
function path.exists(s) end

---@param s string
function path.mkdir(s) end

--- get detail info for a path
---@param s string
---@return PathInfo
function path.get_path_info(s) end

--- get current working dir
---@return string
function path.working_dir() end

--- get the folder of the executable
---@return string
function path.source_base_dir() end

--- return the absolute folder of path
---@param s string
---@return string
function path.absolute(s) end

--- glob with pattern
---@param p string @glob pattern
---@string[]
function path.glob(p) end
