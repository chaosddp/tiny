---@class PathInfo
---@field is_dir    boolean @if the path is a dir
---@field is_file   boolean @if the path is a file
---@field exists    boolean @if the path exists
---@field extension string? @extension of the path if it is a file
---@field file_name string? @file name (with extension) of the path if it is a file
---@field base_name string? @file base name (without extension) of the path if it is a file
---@field parent    string? @parent of the path
local PathInfo = {}

---@class Path
local Path = {}

--- check if the path is a file
---@param s string @path to check
---@return boolean
function Path.is_file(s) end

--- check if the path is a dir
---@param s string @path to check
---@return boolean
function Path.is_dir(s) end

--- check if path exist
---@param s string @path to check
---@return boolean
function Path.exists(s) end

--- get detail info for a path
---@param s string
---@return PathInfo
function Path.get_path_info(s) end

---@type Path
path = {}
