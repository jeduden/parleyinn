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
starts with a real place, `docs/x.md:12`, never `path:line` (a whole file takes
`docs/x.md:1`), then its dash and `if not addressed:` or `request:`; bare, in no
backticks, nothing after its ids. A fine row gets no line, none ending `no
finding`, `none`, `fine` or `sound`. `revised: <full finding line>` replaces the
one at its place (withdraw a moved one, then state it); `withdrawn: x.md:12 —
<why>` removes one. Last message: `outcome: findings|none — <sentence>`.
