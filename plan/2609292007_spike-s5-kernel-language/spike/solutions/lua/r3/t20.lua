local evs = parinn.events()
table.sort(evs, function(x, y)
  if x.writer ~= y.writer then return x.writer < y.writer end
  return x.seq < y.seq
end)
local best = nil
local prev = nil
for _, e in ipairs(evs) do
  if prev ~= nil and prev.writer == e.writer then
    local gap = e.ts - prev.ts
    if best == nil or gap > best[3] then
      best = { e.writer, e.seq, gap }
    end
  end
  prev = e
end
return { best }
