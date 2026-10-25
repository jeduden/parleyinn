local counts = {}
for _, e in ipairs(parinn.events()) do
  if e.kind == "tool_call" and e.tool == "Read" and e.args ~= nil then
    local ok, a = pcall(json.decode, e.args)
    if ok and type(a) == "table" and a.file_path ~= nil then
      local p = a.file_path
      counts[p] = (counts[p] or 0) + 1
    end
  end
end
local rows = {}
for p, c in pairs(counts) do
  rows[#rows + 1] = { p, c }
end
table.sort(rows, function(a, b)
  if a[2] ~= b[2] then
    return a[2] > b[2]
  end
  return a[1] < b[1]
end)
local out = {}
for i = 1, math.min(5, #rows) do
  out[i] = rows[i]
end
return out
