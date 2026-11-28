# Review attestation: checks and review in the thread's own sandbox

Date: 26 to 28 November 2026. Two pages, kept as they were published, each a
self-contained HTML file to open in a browser.

- [status-quo.html](status-quo.html): the research. How the industry splits
  review between the agent's sandbox, CI and the merge gate, which attestation
  building blocks exist, what can go wrong, and where this repository's gate
  stood on 26 November. It proposes no design.
- [design-options.html](design-options.html): the design, fifth draft. A
  thread's own sandbox checks and reviews each commit and signs a receipt; a
  verify job on GitHub, code only, checks the receipt; main's CI and a second
  review on the runner look again after the merge. It weighs one sandbox
  against two gain by gain, models the threats, lists the ways to detect a
  false receipt, and compares the throughput of each approach.

In ParleyInn's domain model the pages' "sandbox" is an ephemeral node,
and their "receipt" is not a CI attestation: no enrolled CI key signs
it. Plan 2611280904 settles its place in the model.

## What was decided

On 28 November the stakeholder chose option 1 of the design, one sandbox per
thread, and set its form: ParleyInn provides the check program as subcommands
of the one `parinn` binary and the verification as a GitHub action shipped with
each release. This repository uses both from an earlier release locked in
`mise.lock`, and other projects take them over the same way.

[Plan 2611280904](../../../plan/2611280904_review-receipts/plan.md) carries
that out.

## Limits

Nothing on either page was built. The figures are the pages' own: measured in
one web sandbox and from GitHub's Actions API on 26 to 28 November, or
estimated from those parts, and each is labelled. The ruling on ENG-21 that
the design asks for is not made yet.
