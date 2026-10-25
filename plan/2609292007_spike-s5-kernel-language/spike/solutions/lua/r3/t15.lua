local evs = parinn.events()
local function args_of(e)
  if type(e.args) ~= "string" then return nil end
  local ok, v = pcall(json.decode, e.args)
  if ok and type(v) == "table" then return v end
  return nil
end
local result_status = {}
for _, e in ipairs(evs) do
  if e.kind == "tool_result" and e.call_id ~= nil and result_status[e.call_id] == nil then
    result_status[e.call_id] = e.status
  end
end
local runs, fails, cmds = {}, {}, {}
for _, e in ipairs(evs) do
  if e.kind == "tool_call" and e.tool == "Bash" then
    local a = args_of(e)
    local c = a and a.command
    if c ~= nil then
      if runs[c] == nil then
        runs[c] = 0
        fails[c] = 0
        cmds[#cmds + 1] = c
      end
      runs[c] = runs[c] + 1
      if e.call_id ~= nil and result_status[e.call_id] == "error" then
        fails[c] = fails[c] + 1
      end
    end
  end
end
local rows = {}
for _, c in ipairs(cmds) do
  if fails[c] > 0 then
    rows[#rows + 1] = { c, runs[c], fails[c] }
  end
end
table.sort(rows, function(x, y)
  if x[3] ~= y[3] then return x[3] > y[3] end
  return x[1] < y[1]
end)
return rows
