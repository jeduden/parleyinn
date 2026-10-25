local errors = {}
local results = {}
for _, e in ipairs(parinn.events()) do
  if e.kind == "tool_result" and e.tool ~= nil then
    results[e.tool] = (results[e.tool] or 0) + 1
    if e.status == "error" then
      errors[e.tool] = (errors[e.tool] or 0) + 1
    end
  end
end

local rows = {}
for tool, r in pairs(results) do
  local err = errors[tool] or 0
  rows[#rows + 1] = { tool, err, r, err / r }
end
table.sort(rows, function(a, b)
  if a[4] ~= b[4] then return a[4] > b[4] end
  return a[1] < b[1]
end)
return rows
