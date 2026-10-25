local evs = parinn.events()
local days = {}
local n = 0
for _, e in ipairs(evs) do
  if e.kind == "message" and e.provenance == "user" then
    local ts = math.tointeger(e.ts) or math.floor(e.ts)
    local d = ts // 86400
    if not days[d] then
      days[d] = true
      n = n + 1
    end
  end
end
return n
