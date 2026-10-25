local events = parinn.events()
table.sort(events, function(x, y)
  if x.writer ~= y.writer then return x.writer < y.writer end
  return x.seq < y.seq
end)
local function path_of(e)
  if e.args == nil then return nil end
  local ok, a = pcall(json.decode, e.args)
  if ok and type(a) == "table" then return a.file_path end
  return nil
end
local read = {}
local n = 0
for _, e in ipairs(events) do
  if e.kind == "tool_call" then
    local w = e.writer
    if read[w] == nil then read[w] = {} end
    if e.tool == "Read" then
      local p = path_of(e)
      if p ~= nil then read[w][p] = true end
    elseif e.tool == "Edit" then
      local p = path_of(e)
      if p == nil or not read[w][p] then
        n = n + 1
      end
    end
  end
end
return n
