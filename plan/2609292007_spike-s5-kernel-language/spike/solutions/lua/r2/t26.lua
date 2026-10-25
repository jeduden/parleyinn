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
local sessions = 0
for _, list in pairs(byrun) do
  table.sort(list, function(a, b)
    if a.seq ~= b.seq then return a.seq < b.seq end
    return a.writer < b.writer
  end)
  for i, e in ipairs(list) do
    if i == 1 or e.ts - list[i - 1].ts > 1800 then
      sessions = sessions + 1
    end
  end
end
return sessions
