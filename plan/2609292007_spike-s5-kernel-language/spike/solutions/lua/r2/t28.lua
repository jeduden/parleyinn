local events = parinn.events()
table.sort(events, function(a, b)
  if a.writer ~= b.writer then return a.writer < b.writer end
  return a.seq < b.seq
end)
local counts = {}
local writer = nil
local in_turn = false
local turn_run = nil
local turn_err = false

local function close_turn()
  if in_turn and turn_err and turn_run then
    counts[turn_run] = (counts[turn_run] or 0) + 1
  end
  in_turn = false
  turn_run = nil
  turn_err = false
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
    turn_err = true
  end
end
close_turn()

local rows = {}
for run, n in pairs(counts) do
  rows[#rows + 1] = { run, n }
end
table.sort(rows, function(a, b) return a[1] < b[1] end)
return rows
