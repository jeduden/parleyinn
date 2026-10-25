local function decode_args(e)
  if type(e.args) ~= "string" then return nil end
  local ok, v = pcall(json.decode, e.args)
  if ok and type(v) == "table" then return v end
  return nil
end

local counts = {}
for _, e in ipairs(parinn.events()) do
  if e.kind == "tool_call" and e.tool == "Read" then
    local a = decode_args(e)
    local p = a and a.file_path
    if type(p) == "string" then
      counts[p] = (counts[p] or 0) + 1
    end
  end
end

local rows = {}
for p, c in pairs(counts) do
  rows[#rows + 1] = { p, c }
end
table.sort(rows, function(a, b)
  if a[2] ~= b[2] then return a[2] > b[2] end
  return a[1] < b[1]
end)

local out = {}
for i = 1, math.min(5, #rows) do
  out[i] = rows[i]
end
return out
