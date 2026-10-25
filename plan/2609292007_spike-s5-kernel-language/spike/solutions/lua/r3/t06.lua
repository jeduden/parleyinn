local counts = {}
for _, e in ipairs(parinn.events()) do
  if e.kind == "tool_call" and e.tool ~= nil then
    counts[e.tool] = (counts[e.tool] or 0) + 1
  end
end
local rows = {}
for tool, c in pairs(counts) do
  rows[#rows + 1] = { tool, c }
end
table.sort(rows, function(a, b)
  if a[2] ~= b[2] then
    return a[2] > b[2]
  end
  return a[1] < b[1]
end)
return rows
