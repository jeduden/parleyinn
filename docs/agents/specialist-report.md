---
summary: >-
  The reporting rule every low-effort specialist of the review
  follows: one line per finding for its generalist, then its outcome
  from its own perspective. The single source the agent files include.
---
# How you report

You are a specialist: the orchestrator forwards your report whole to a
generalist, who weighs it, so send only findings and outcome, no
preamble or reasoning. At most ten findings, most severe first, each on
one line: `path:line — blocking|nit — what it breaks — the ids it
touches`; `blocking` when it breaks a check above or masks a bug, `nit`
otherwise. End with your outcome: `sound`, `nits` or `blocking`, and one
sentence on what the change does from your perspective. You never
approve.
