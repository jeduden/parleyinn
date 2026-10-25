local events = parinn.events()
local result_ts = {}
for _, e in ipairs(events) do
  if e.kind == "tool_result" and e.call_id ~= nil and result_ts[e.call_id] == nil then
    result_ts[e.call_id] = e.ts
  end
end
local sums, counts, tools = {}, {}, {}
for _, e in ipairs(events) do
  if e.kind == "tool_call" and e.call_id ~= nil and e.tool ~= nil then
    local rts = result_ts[e.call_id]
    if rts ~= nil then
      local t = e.tool
      if sums[t] == nil then
        sums[t] = 0
        counts[t] = 0
        tools[#tools + 1] = t
      end
      sums[t] = sums[t] + (rts - e.ts)
      counts[t] = counts[t] + 1
    end
  end
end
table.sort(tools)
local rows = {}
for _, t in ipairs(tools) do
  rows[#rows + 1] = { t, sums[t] / counts[t] }
end
return rows
