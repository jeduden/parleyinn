local count = 0
for _, e in ipairs(parinn.events()) do
  if e.kind == "tool_call" and e.tool == "Bash" then
    count = count + 1
  end
end
return count
