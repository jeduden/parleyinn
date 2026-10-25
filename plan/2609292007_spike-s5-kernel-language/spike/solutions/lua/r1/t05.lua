local count = 0
for _, e in ipairs(parinn.events()) do
  if e.kind == "message" and e.provenance == "user" and e.text then
    if string.find(string.lower(e.text), "test", 1, true) then
      count = count + 1
    end
  end
end
return count
