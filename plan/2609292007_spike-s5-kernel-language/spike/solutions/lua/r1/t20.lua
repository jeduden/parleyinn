local evs = parinn.events()
table.sort(evs, function(a, b)
  if a.writer ~= b.writer then return a.writer < b.writer end
  return a.seq < b.seq
end)
local best = nil
for i = 2, #evs do
  local p, e = evs[i - 1], evs[i]
  if p.writer == e.writer then
    local g = e.ts - p.ts
    if best == nil or g > best[3]
      or (g == best[3] and (e.writer < best[1] or (e.writer == best[1] and e.seq < best[2]))) then
      best = { e.writer, e.seq, g }
    end
  end
end
return { best }
