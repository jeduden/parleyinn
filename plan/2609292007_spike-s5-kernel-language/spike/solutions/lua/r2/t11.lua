local counts, maxseq, writers = {}, {}, {}
for _, e in ipairs(parinn.events()) do
  local w = e.writer
  if counts[w] == nil then
    counts[w] = 0
    maxseq[w] = e.seq
    writers[#writers + 1] = w
  end
  counts[w] = counts[w] + 1
  if e.seq > maxseq[w] then maxseq[w] = e.seq end
end
table.sort(writers)
local rows = {}
for _, w in ipairs(writers) do
  rows[#rows + 1] = { w, counts[w], maxseq[w] }
end
return rows
