local events = parinn.events()
local result_status = {}
for _, e in ipairs(events) do
  if e.kind == "tool_result" and e.call_id ~= nil and result_status[e.call_id] == nil then
    result_status[e.call_id] = e.status
  end
end
local runs, failures, cmds = {}, {}, {}
for _, e in ipairs(events) do
  if e.kind == "tool_call" and e.tool == "Bash" and e.args ~= nil then
    local ok, a = pcall(json.decode, e.args)
    if ok and type(a) == "table" and a.command ~= nil then
      local c = a.command
      if runs[c] == nil then
        runs[c] = 0
        failures[c] = 0
        cmds[#cmds + 1] = c
      end
      runs[c] = runs[c] + 1
      if e.call_id ~= nil and result_status[e.call_id] == "error" then
        failures[c] = failures[c] + 1
      end
    end
  end
end
local rows = {}
for _, c in ipairs(cmds) do
  if failures[c] > 0 then
    rows[#rows + 1] = { c, runs[c], failures[c] }
  end
end
table.sort(rows, function(x, y)
  if x[3] ~= y[3] then return x[3] > y[3] end
  return x[1] < y[1]
end)
return rows
