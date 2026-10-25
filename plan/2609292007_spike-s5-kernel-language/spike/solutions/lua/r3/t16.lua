local evs = parinn.events()
table.sort(evs, function(x, y)
  if x.writer ~= y.writer then return x.writer < y.writer end
  return x.seq < y.seq
end)
local function args_of(e)
  if type(e.args) ~= "string" then return nil end
  local ok, v = pcall(json.decode, e.args)
  if ok and type(v) == "table" then return v end
  return nil
end
local read = {}
local n = 0
for _, e in ipairs(evs) do
  if e.kind == "tool_call" then
    local w = e.writer
    if read[w] == nil then read[w] = {} end
    if e.tool == "Read" then
      local a = args_of(e)
      if a and a.file_path ~= nil then read[w][a.file_path] = true end
    elseif e.tool == "Edit" then
      local a = args_of(e)
      local p = a and a.file_path
      if p == nil or not read[w][p] then
        n = n + 1
      end
    end
  end
end
return n
