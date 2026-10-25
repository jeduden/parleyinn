local counts = {}
for _, e in ipairs(parinn.events()) do
  if e.kind == "tool_call" and e.tool == "Edit" and e.args then
    local a = json.decode(e.args)
    if type(a) == "table" and type(a.file_path) == "string" then
      local ext = string.match(a.file_path, "%.([^%.]*)$")
      if ext then
        counts[ext] = (counts[ext] or 0) + 1
      end
    end
  end
end
local rows = {}
for ext, n in pairs(counts) do
  rows[#rows + 1] = { ext, n }
end
table.sort(rows, function(x, y)
  if x[2] ~= y[2] then
    return x[2] > y[2]
  end
  return x[1] < y[1]
end)
return rows
