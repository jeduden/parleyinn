local evs = parinn.events()
table.sort(evs, function(a, b)
  if a.writer ~= b.writer then return a.writer < b.writer end
  return a.seq < b.seq
end)
local counts = {}
local curWriter = nil
local inTurn = false
local turnRun = nil
local turnErr = false

local function closeTurn()
  if inTurn and turnErr and turnRun ~= nil then
    counts[turnRun] = (counts[turnRun] or 0) + 1
  end
  inTurn = false
  turnRun = nil
  turnErr = false
end

for _, e in ipairs(evs) do
  if e.writer ~= curWriter then
    closeTurn()
    curWriter = e.writer
  end
  if e.kind == "message" and e.provenance == "user" then
    closeTurn()
    inTurn = true
    turnRun = e.run
    turnErr = false
  end
  if inTurn and e.kind == "tool_result" and e.status == "error" then
    turnErr = true
  end
end
closeTurn()

local rows = {}
for run, n in pairs(counts) do
  rows[#rows + 1] = { run, n }
end
table.sort(rows, function(a, b) return a[1] < b[1] end)
return rows
