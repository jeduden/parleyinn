local seen = {}
local n = 0
for _, e in ipairs(parinn.events()) do
  if e.kind == "tool_call" and (e.tool == "Edit" or e.tool == "Write") and e.args ~= nil then
    local ok, a = pcall(json.decode, e.args)
    if ok and type(a) == "table" then
      local p = a.file_path
      if p ~= nil and not seen[p] then
        seen[p] = true
        n = n + 1
      end
    end
  end
end
return n
