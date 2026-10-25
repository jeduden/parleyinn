local evs = parinn.events()
local lo, hi, runs = {}, {}, {}
for _, e in ipairs(evs) do
  local r = e.run
  if r ~= nil then
    if lo[r] == nil then
      lo[r] = e.ts
      hi[r] = e.ts
      runs[#runs + 1] = r
    else
      if e.ts < lo[r] then lo[r] = e.ts end
      if e.ts > hi[r] then hi[r] = e.ts end
    end
  end
end
local rows = {}
for _, r in ipairs(runs) do
  rows[#rows + 1] = { r, hi[r] - lo[r] }
end
table.sort(rows, function(x, y)
  if x[2] ~= y[2] then return x[2] > y[2] end
  return x[1] < y[1]
end)
return rows
