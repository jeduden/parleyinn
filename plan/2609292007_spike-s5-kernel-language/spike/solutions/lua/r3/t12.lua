local evs = parinn.events()
local result_ts = {}
for _, e in ipairs(evs) do
  if e.kind == "tool_result" and e.call_id ~= nil and result_ts[e.call_id] == nil then
    result_ts[e.call_id] = e.ts
  end
end
local sum, n, tools = {}, {}, {}
for _, e in ipairs(evs) do
  if e.kind == "tool_call" and e.call_id ~= nil and result_ts[e.call_id] ~= nil then
    local t = e.tool
    if sum[t] == nil then
      sum[t] = 0
      n[t] = 0
      tools[#tools + 1] = t
    end
    sum[t] = sum[t] + (result_ts[e.call_id] - e.ts)
    n[t] = n[t] + 1
  end
end
table.sort(tools)
local rows = {}
for _, t in ipairs(tools) do
  rows[#rows + 1] = { t, sum[t] / n[t] }
end
return rows
