local byRun = {}
for _, e in ipairs(parinn.events()) do
  if e.run ~= nil then
    local list = byRun[e.run]
    if list == nil then
      list = {}
      byRun[e.run] = list
    end
    list[#list + 1] = e
  end
end
local sessions = 0
for _, list in pairs(byRun) do
  table.sort(list, function(a, b) return a.seq < b.seq end)
  local prevTs = nil
  for _, e in ipairs(list) do
    if prevTs == nil or e.ts - prevTs > 1800 then
      sessions = sessions + 1
    end
    prevTs = e.ts
  end
end
return sessions
