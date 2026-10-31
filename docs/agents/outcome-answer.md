---
summary: >-
  How a specialist of the review states its findings and answers: no
  file, each finding sent as a message the moment it is found, a
  revision or withdrawal when it changes its mind, and a last message
  of the outcome line alone. The single source the report sections and
  agent files include.
---
# How you answer

No file: your run log is your report. Right after you read your diff, send one
message with a line per finding it shows, and each later one at once. A line
starts with a real place, `docs/x.md:12`, never `path:line`: a whole file, row
or change takes its nearest line, or `docs/x.md:1`. Then its dash and `if not
addressed:` or `request:`, nothing else. `revised: <new line>`, one full finding
line alone, replaces the one at its place (withdraw a moved one, then state it);
`withdrawn: docs/x.md:12 — <why>` removes one you were wrong about, never what
you only checked. Last message: `outcome: findings|none — <sentence>`.
