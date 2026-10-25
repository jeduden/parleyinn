local byrun = {}
for _, e in ipairs(parinn.events()) do
  if e.run then
    local list = byrun[e.run]
    if not list then
      list = {}
      byrun[e.run] = list
    end
    list[#list + 1] = e
  end
end
local runs = {}
for r in pairs(byrun) do
  runs[#runs + 1] = r
end
table.sort(runs)
local rows = {}
for _, r in ipairs(runs) do
  local list = byrun[r]
  table.sort(list, function(x, y)
    if x.writer ~= y.writer then
      return x.writer < y.writer
    end
    return x.seq < y.seq
  end)
  local sum = 0
  for _, e in ipairs(list) do
    sum = sum + (e.tokens or 0)
    if sum >= 50000 then
      rows[#rows + 1] = { r, e.seq }
      break
    end
  end
end
return rows
