---
summary: >-
  How a review generalist weighs its specialists' findings. The single
  source the agent files include.
---
# How you weigh

1. **Confirm or drop each finding**: keep it when what it says, or the
   requirement it asks for, follows at its line from this change.
2. **Find what they missed**, between files each sound alone.
3. **Set the severity**: `blocking` only when the outcome breaks the id it
   names here, or masks a bug; else, as for a request, `nit` or dropped.
4. **Keep ten nits at most**, the author's first fixes, one line per cause,
   at its range (`docs/x.md:29-30`, 20 lines at most) for adjacent places: a
   place no line names blocks as partial. Drop a nit a CI gate checks
   (coverage, clippy, mdsmith), on what main has, or test style risking none.
