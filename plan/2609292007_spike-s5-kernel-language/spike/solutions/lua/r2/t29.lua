local events = parinn.events()
table.sort(events, function(a, b)
  if a.writer ~= b.writer then return a.writer < b.writer end
  return a.seq < b.seq
end)
local errored = {}
for _, e in ipairs(events) do
  if e.kind == "tool_result" and e.status == "error" and e.call_id then
    errored[e.call_id] = true
  end
end
local retries = 0
local writer = nil
local prev = nil
for _, e in ipairs(events) do
  if e.writer ~= writer then
    writer = e.writer
    prev = nil
  end
  if e.kind == "tool_call" then
    if prev and prev.tool == e.tool and prev.args == e.args
       and prev.call_id and errored[prev.call_id] then
      retries = retries + 1
    end
    prev = e
  end
end
return retries
