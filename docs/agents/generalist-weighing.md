---
summary: >-
  How a review generalist weighs its specialists' findings. The single
  source the agent files include.
---
# How you weigh

1. **Confirm or drop each specialist finding**: keep it only when what it
   says happens, or the requirement it asks for, follows at its line from
   this change. Expect false alarms; say why you drop one.
2. **Find what they missed**, between files each sound alone.
3. **Set the severity**: `blocking` only when the outcome breaks the id it
   names here, or masks a bug; else, as for a request, `nit` or dropped.
4. **Keep ten nits at most**, the author's first fixes, one line per cause
   naming every place. Drop a nit a CI gate checks (coverage, clippy,
   mdsmith), one on what main has, or test style risking no behaviour.
