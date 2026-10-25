local counts = {}
for _, e in ipairs(parinn.events()) do
  if e.kind == "tool_result" and e.tool == "Bash" and e.status == "error" and e.text then
    local c = re.capture(e.text, "exit code (-?\\d+)", 1)
    if c then
      local code = math.tointeger(tonumber(c))
      if code ~= nil then
        counts[code] = (counts[code] or 0) + 1
      end
    end
  end
end
local codes = {}
for code in pairs(counts) do
  codes[#codes + 1] = code
end
table.sort(codes)
local rows = {}
for _, code in ipairs(codes) do
  rows[#rows + 1] = { code, counts[code] }
end
return rows
