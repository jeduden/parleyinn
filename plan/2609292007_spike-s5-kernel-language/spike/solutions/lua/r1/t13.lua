local evs = parinn.events()
local has = {}
for _, e in ipairs(evs) do
  if e.kind == "tool_result" and e.call_id ~= nil then
    has[e.call_id] = true
  end
end
local c = 0
for _, e in ipairs(evs) do
  if e.kind == "tool_call" then
    if e.call_id == nil or not has[e.call_id] then
      c = c + 1
    end
  end
end
return c
