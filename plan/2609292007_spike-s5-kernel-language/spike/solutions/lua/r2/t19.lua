local days = {}
local n = 0
for _, e in ipairs(parinn.events()) do
  if e.kind == "message" and e.provenance == "user" then
    local d = math.floor(e.ts / 86400)
    d = math.tointeger(d) or d
    if not days[d] then
      days[d] = true
      n = n + 1
    end
  end
end
return n
