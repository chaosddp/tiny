---@class RequestOptions
---@field url      string
---@field body?    string | table<string, any>
---@field headers? table<string, string>

---@class LuaHttpClient
local LuaHttpClient = {}

--- post data to url
---@param method    "get" | "post" | "put"      :"delete" @http request method
---@param url       string                      @url to send request
---@param body?     string | table<string, any> @if it is a string, then send as plain text, if it is a table, then send as json content
---@param headers?  table<string, string>?      @customize headers to send
---@param callback? fun(line: string): void     @callback with content of the stream event, called line by line
---@return boolean
---@return any
function LuaHttpClient:send(method, url, body, headers, callback) end

--- create a new instance of http client
---@return LuaHttpClient
function HttpClient() end
