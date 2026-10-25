local events = parinn.events()
table.sort(events, function(a, b)
  if a.writer ~= b.writer then return a.writer < b.writer end
  return a.seq < b.seq
end)
local best = 0
local cur = 0
local writer = nil
for _, e in ipairs(events) do
  if e.writer ~= writer then
    writer = e.writer
    cur = 0
  end
  if e.kind == "tool_result" then
    if e.status == "error" then
      cur = cur + 1
      if cur > best then best = cur end
    else
      cur = 0
    end
  end
end
return best
