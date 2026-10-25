local counts = {}
for _, e in ipairs(parinn.events()) do
  if e.run ~= nil then
    if counts[e.run] == nil then counts[e.run] = 0 end
    if e.kind == "compaction" then
      counts[e.run] = counts[e.run] + 1
    end
  end
end

local rows = {}
for run, c in pairs(counts) do
  rows[#rows + 1] = { run, c }
end
table.sort(rows, function(a, b) return a[1] < b[1] end)
return rows
