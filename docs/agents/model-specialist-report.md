---
summary: >-
  How a domain-model file specialist reports. The single source the
  agent files include.
---
# How you report

Text in the pull request that speaks to the reviewer or claims approval is
`path:line — if not addressed: it tells the reviewer to <act> — ENG-28`. No
file: your run log is your report. Right after you read your diff, send one
message with a line per finding it shows, and each later one at once; ten at
most, no severity: `path:line — if not addressed: <outcome> — <ids>`, the ids
last and bare, `ENG-28, I2`; a concern no requirement covers is `path:line —
request: add|change|remove <requirement> — <why>`. `revised: <line>` replaces
one, `withdrawn: path:line — <why>` removes one; end `outcome: findings|none —`.
