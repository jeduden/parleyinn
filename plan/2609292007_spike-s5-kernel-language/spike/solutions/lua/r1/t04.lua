local seen = {}
local count = 0
for _, e in ipairs(parinn.events()) do
  if e.kind == "tool_call" and (e.tool == "Edit" or e.tool == "Write") and e.args then
    local ok, a = pcall(json.decode, e.args)
    if ok and type(a) == "table" and a.file_path ~= nil then
      local p = a.file_path
      if not seen[p] then
        seen[p] = true
        count = count + 1
      end
    end
  end
end
return count
