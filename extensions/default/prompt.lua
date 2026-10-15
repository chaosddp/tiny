
local prompts = {}

function prompts:prompt(cur_prompt)
    return cur_prompt .. "\n\nIf you do not know the answer, DO NOT guess, just tell the user your do not know."
end

return prompts