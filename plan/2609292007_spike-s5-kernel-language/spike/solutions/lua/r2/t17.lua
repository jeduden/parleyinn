local counts = {}
for _, e in ipairs(parinn.events()) do
  if e.kind == "message" and e.provenance == "user" then
    local ts = math.tointeger(e.ts) or math.floor(e.ts)
    local h = (ts // 3600) % 24
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
