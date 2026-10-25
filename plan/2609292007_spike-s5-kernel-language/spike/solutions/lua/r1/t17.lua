local evs = parinn.events()
local counts = {}
for _, e in ipairs(evs) do
  if e.kind == "message" and e.provenance == "user" then
    local h = math.floor((e.ts % 86400) / 3600)
    h = math.tointeger(h) or h
    counts[h] = (counts[h] or 0) + 1
  end
end
local rows = {}
for h = 0, 23 do
  if counts[h] ~= nil then
    rows[#rows + 1] = { h, counts[h] }
  end
end
return rows
