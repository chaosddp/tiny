---@diagnostic disable: missing-return
---@class LuaHttpClient
local LuaHttpClient = {}

--- send data to url
---@param method    "get" | "post" | "put" | "delete"
---@param url       string                            @url to send request
---@param body?     string | table<string, any>       @if it is a string, then send as plain text, if it is a table, then send as json content
---@param headers?  table<string, string>             @customize headers to send
---@param callback? fun(line: string): void           @callback with content of the reponse, called line by line
---@return (string | string[])? @string list if it is a SSE stream, or a string
function LuaHttpClient:send(method, url, body, headers, callback) end

--- create a new instance of http client
---@return LuaHttpClient
function HttpClient() end
