local events = parinn.events()
local result_ts = {}
for _, e in ipairs(events) do
  if e.kind == "tool_result" and e.call_id ~= nil and result_ts[e.call_id] == nil then
    result_ts[e.call_id] = e.ts
  end
end
local best_id, best_d = nil, nil
for _, e in ipairs(events) do
  if e.kind == "tool_call" and e.call_id ~= nil then
    local rts = result_ts[e.call_id]
    if rts ~= nil then
      local d = rts - e.ts
      if best_d == nil or d > best_d or (d == best_d and e.call_id < best_id) then
        best_d = d
        best_id = e.call_id
      end
    end
  end
end
return best_id
