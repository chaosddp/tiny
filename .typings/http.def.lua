---@class LuaHttpClient
local LuaHttpClient = {}

--- post data to url
---@param url      string                      @url to send request
---@param body     string | table<string, any> @if it is a string, then send as plain text, if it is a table, then send as json content
---@param headers  table<string, string>?      @customize headers to send
---@param callback fun(line: string): void     @callback with content of the reponse, called line by line
function LuaHttpClient:post(url, body, callback, headers) end

--- create a new instance of http client
---@return LuaHttpClient
function HttpClient() end
