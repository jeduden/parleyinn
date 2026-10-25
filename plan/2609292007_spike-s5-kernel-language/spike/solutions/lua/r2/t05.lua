local n = 0
for _, e in ipairs(parinn.events()) do
  if e.kind == "message" and e.provenance == "user" and type(e.text) == "string" then
    if string.find(string.lower(e.text), "test", 1, true) then
      n = n + 1
    end
  end
end
return n
