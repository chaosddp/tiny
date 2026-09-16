-- local success, b = pcall(require, "b")

-- tiny.test()

-- tiny.a.b.test()

-- print(tiny.a.b.pi)

-- print(package.path)

-- if success then
--     print(b.version)
-- end

-- function sum(a, b)
--     return a + b
-- end


-- local client = tiny.http.newClient()

-- if client then 
--     local resp = client:get("https://www.baidu.com")

--     print(resp:status())

--     print(resp:text())
-- end

function tools()

end

function tiny.conf(t)
    t.chat.provider = "openai" -- use openai compatible provider
    t.chat.model = "qwen3.5"
    t.chat.base_url = "http://localhost:11434/v1"
    t.chat.api_key = "ollama"

    t.chat.max_tokens = 10240000
    
    t.chat.thinking.type = "enabled"
    t.chat.thinking.budget_tokens = 8192

    t.chat.reasoning_effort = "low" -- low, medium, hight, any other string

    t.system_prompt = "myprompt"

    t.tools = tools()
end

-- called each loop cycle, usage:
-- 1. cancel current loop
-- 2. retry from beginning
function tiny.on_inner_loop_count(n)

end

function tiny.on_response_error(e)

end

function tiny.before_chat(ctx)

end

function tiny.after_chat(ctx)

end

