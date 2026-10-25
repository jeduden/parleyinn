local sums = {}
for _, e in ipairs(parinn.events()) do
  if e.kind == "message" and e.provenance == "assistant" and e.run ~= nil then
    sums[e.run] = (sums[e.run] or 0) + (e.tokens or 0)
  end
end

local rows = {}
for run, t in pairs(sums) do
  rows[#rows + 1] = { run, t }
end
table.sort(rows, function(a, b)
  if a[2] ~= b[2] then return a[2] > b[2] end
  return a[1] < b[1]
end)

local out = {}
for i = 1, math.min(3, #rows) do
  out[i] = rows[i]
end
return out
