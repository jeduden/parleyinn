local evs = parinn.events()
local results = {}
for _, e in ipairs(evs) do
  if e.kind == "tool_result" and e.call_id ~= nil and results[e.call_id] == nil then
    results[e.call_id] = e
  end
end
local best_id, best_d = nil, nil
for _, e in ipairs(evs) do
  if e.kind == "tool_call" and e.call_id ~= nil then
    local r = results[e.call_id]
    if r ~= nil then
      local d = r.ts - e.ts
      if best_d == nil or d > best_d or (d == best_d and e.call_id < best_id) then
        best_d = d
        best_id = e.call_id
      end
    end
  end
end
return best_id
