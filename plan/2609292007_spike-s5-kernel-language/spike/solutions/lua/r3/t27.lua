local evs = parinn.events()
table.sort(evs, function(a, b)
  if a.writer ~= b.writer then return a.writer < b.writer end
  return a.seq < b.seq
end)
local active = {}
for _, e in ipairs(evs) do
  if e.kind == "pin" and e.args ~= nil then
    local a = json.decode(e.args)
    if type(a) == "table" and a.pin_id ~= nil then
      if a.action == "add" then
        active[a.pin_id] = true
      elseif a.action == "remove" then
        active[a.pin_id] = nil
      end
    end
  end
end
local out = {}
for id in pairs(active) do
  out[#out + 1] = id
end
table.sort(out)
return out
