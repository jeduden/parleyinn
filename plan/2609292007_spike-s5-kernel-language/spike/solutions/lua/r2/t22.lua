local events = parinn.events()
local call_tool = {}
for _, e in ipairs(events) do
  if e.kind == "tool_call" and e.call_id then
    call_tool[e.call_id] = e.tool
  end
end
local counts = {}
for _, e in ipairs(events) do
  if e.kind == "tool_result" and e.status == "error" then
    local tool = e.tool
    if tool == nil and e.call_id then tool = call_tool[e.call_id] end
    if tool == "Bash" and e.text then
      local c = re.capture(e.text, "exit code\\s*(-?\\d+)")
      if c then
        local code = math.tointeger(tonumber(c))
        counts[code] = (counts[code] or 0) + 1
      end
    end
  end
end
local rows = {}
for code, n in pairs(counts) do
  rows[#rows + 1] = { code, n }
end
table.sort(rows, function(a, b) return a[1] < b[1] end)
return rows
