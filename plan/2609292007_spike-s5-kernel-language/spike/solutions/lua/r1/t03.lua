local count = 0
for _, e in ipairs(parinn.events()) do
  if e.kind == "tool_result" and e.status == "error" then
    count = count + 1
  end
end
return count
