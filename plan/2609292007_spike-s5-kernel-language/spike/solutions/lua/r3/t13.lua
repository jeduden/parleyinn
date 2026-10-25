local evs = parinn.events()
local has_result = {}
for _, e in ipairs(evs) do
  if e.kind == "tool_result" and e.call_id ~= nil then
    has_result[e.call_id] = true
  end
end
local n = 0
for _, e in ipairs(evs) do
  if e.kind == "tool_call" and (e.call_id == nil or not has_result[e.call_id]) then
    n = n + 1
  end
end
return n
