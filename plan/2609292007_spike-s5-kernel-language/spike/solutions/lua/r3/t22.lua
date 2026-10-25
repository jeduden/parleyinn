local counts = {}
for _, e in ipairs(parinn.events()) do
  if e.kind == "tool_result" and e.tool == "Bash" and e.status == "error" and e.text ~= nil then
    local c = re.capture(e.text, "exit code (-?\\d+)", 1)
    if c ~= nil then
      local code = math.tointeger(tonumber(c))
      counts[code] = (counts[code] or 0) + 1
    end
  end
end
local rows = {}
for code, n in pairs(counts) do
  rows[#rows + 1] = { code, n }
end
table.sort(rows, function(a, b) return a[1] < b[1] end)
return rows
