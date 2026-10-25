local evs = parinn.events()
table.sort(evs, function(a, b)
  if a.writer ~= b.writer then return a.writer < b.writer end
  return a.seq < b.seq
end)
local function path_of(e)
  if e.args == nil then return nil end
  local ok, a = pcall(json.decode, e.args)
  if ok and type(a) == "table" then return a.file_path end
  return nil
end
local read = {}
local c = 0
for _, e in ipairs(evs) do
  if e.kind == "tool_call" then
    local w = e.writer
    if read[w] == nil then read[w] = {} end
    if e.tool == "Edit" then
      local p = path_of(e)
      if p == nil or not read[w][p] then
        c = c + 1
      end
    elseif e.tool == "Read" then
      local p = path_of(e)
      if p ~= nil then read[w][p] = true end
    end
  end
end
return c
