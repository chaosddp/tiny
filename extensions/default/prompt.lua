local default_prompt = string.format(
    [[
You are a helpful assistant at %s, and all you modification should limit to this folder. 
Use the tools to solve task with following guideline:
- Act, do not explain.
- Do not guess, tell the user if you do not know what to do
- Create a todo list for complex task, finish one by one
- Make sure the output to user is UTF-8 charset if possible
]],
path.working_dir()
)

local prompts = {}

function prompts:prompt(cur_prompt)
    return default_prompt
end

return prompts
