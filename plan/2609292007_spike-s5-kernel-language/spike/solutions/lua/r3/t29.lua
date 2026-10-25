local evs = parinn.events()
table.sort(evs, function(a, b)
  if a.writer ~= b.writer then return a.writer < b.writer end
  return a.seq < b.seq
end)
local errored = {}
for _, e in ipairs(evs) do
  if e.kind == "tool_result" and e.status == "error" and e.call_id ~= nil then
    errored[e.writer .. "\0" .. e.call_id] = true
  end
end
local retries = 0
local curWriter = nil
local prev = nil
for _, e in ipairs(evs) do
  if e.writer ~= curWriter then
    curWriter = e.writer
    prev = nil
  end
  if e.kind == "tool_call" then
    if prev ~= nil and prev.tool == e.tool and prev.args == e.args
      and prev.call_id ~= nil and errored[prev.writer .. "\0" .. prev.call_id] then
      retries = retries + 1
    end
    prev = e
  end
end
return retries
