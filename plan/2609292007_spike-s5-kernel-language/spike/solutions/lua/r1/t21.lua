local total = 0
for _, e in ipairs(parinn.events()) do
  if e.kind == "compaction" and e.args then
    local a = json.decode(e.args)
    if type(a) == "table" and a.trigger == "auto" and a.pre_tokens then
      total = total + a.pre_tokens
    end
  end
end
return total
