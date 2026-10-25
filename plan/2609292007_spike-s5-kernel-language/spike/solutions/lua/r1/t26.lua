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
local sessions = 0
for _, list in pairs(byrun) do
  table.sort(list, function(x, y)
    if x.writer ~= y.writer then
      return x.writer < y.writer
    end
    return x.seq < y.seq
  end)
  local prev = nil
  for _, e in ipairs(list) do
    if prev == nil or e.ts - prev > 1800 then
      sessions = sessions + 1
    end
    prev = e.ts
  end
end
return sessions
