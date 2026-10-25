local events = parinn.events()
table.sort(events, function(x, y)
  if x.writer ~= y.writer then
    return x.writer < y.writer
  end
  return x.seq < y.seq
end)
local counts = {}
local writer = nil
local turn_run = nil
local in_turn = false
local turn_has_error = false

local function close_turn()
  if in_turn and turn_has_error and turn_run then
    counts[turn_run] = (counts[turn_run] or 0) + 1
  end
  in_turn = false
  turn_has_error = false
  turn_run = nil
end

for _, e in ipairs(events) do
  if e.writer ~= writer then
    close_turn()
    writer = e.writer
  end
  if e.kind == "message" and e.provenance == "user" then
    close_turn()
    in_turn = true
    turn_run = e.run
  end
  if in_turn and e.kind == "tool_result" and e.status == "error" then
    turn_has_error = true
  end
end
close_turn()

local runs = {}
for r in pairs(counts) do
  runs[#runs + 1] = r
end
table.sort(runs)
local rows = {}
for _, r in ipairs(runs) do
  rows[#rows + 1] = { r, counts[r] }
end
return rows
