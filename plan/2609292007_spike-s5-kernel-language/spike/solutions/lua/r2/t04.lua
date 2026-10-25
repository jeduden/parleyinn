local function decode_args(e)
  if type(e.args) ~= "string" then return nil end
  local ok, v = pcall(json.decode, e.args)
  if ok and type(v) == "table" then return v end
  return nil
end

local seen = {}
local n = 0
for _, e in ipairs(parinn.events()) do
  if e.kind == "tool_call" and (e.tool == "Edit" or e.tool == "Write") then
    local a = decode_args(e)
    local p = a and a.file_path
    if type(p) == "string" and not seen[p] then
      seen[p] = true
      n = n + 1
    end
  end
end
return n
