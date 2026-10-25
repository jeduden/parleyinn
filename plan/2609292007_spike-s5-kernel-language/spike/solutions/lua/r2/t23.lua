local counts = {}
for _, e in ipairs(parinn.events()) do
  if e.kind == "recall" and e.args then
    local a = json.decode(e.args)
    if type(a) == "table" and type(a.query) == "string" then
      counts[a.query] = (counts[a.query] or 0) + 1
    end
  end
end
local out = {}
for q, n in pairs(counts) do
  if n > 1 then out[#out + 1] = q end
end
table.sort(out)
return out
