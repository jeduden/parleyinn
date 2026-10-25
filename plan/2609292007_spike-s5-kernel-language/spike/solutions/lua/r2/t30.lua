local byrun = {}
for _, e in ipairs(parinn.events()) do
  if e.run then
    local t = byrun[e.run]
    if not t then
      t = {}
      byrun[e.run] = t
    end
    t[#t + 1] = e
  end
end
local rows = {}
for run, list in pairs(byrun) do
  table.sort(list, function(a, b) return a.seq < b.seq end)
  local sum = 0
  for _, e in ipairs(list) do
    sum = sum + (e.tokens or 0)
    if sum >= 50000 then
      rows[#rows + 1] = { run, e.seq }
      break
    end
  end
end
table.sort(rows, function(a, b) return a[1] < b[1] end)
return rows
