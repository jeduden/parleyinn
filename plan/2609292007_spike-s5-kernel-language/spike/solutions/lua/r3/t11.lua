local evs = parinn.events()
local count, maxseq, writers = {}, {}, {}
for _, e in ipairs(evs) do
  local w = e.writer
  if count[w] == nil then
    count[w] = 0
    maxseq[w] = e.seq
    writers[#writers + 1] = w
  end
  count[w] = count[w] + 1
  if e.seq > maxseq[w] then maxseq[w] = e.seq end
end
table.sort(writers)
local rows = {}
for _, w in ipairs(writers) do
  rows[#rows + 1] = { w, count[w], maxseq[w] }
end
return rows
