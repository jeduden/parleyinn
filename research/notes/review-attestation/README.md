# Review attestation: checks and review in the thread's own sandbox

Date: 26 to 28 November 2026. Three pages, kept as they were published, each a
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
- [open-questions.html](open-questions.html): the four questions the
  domain-model agent left open on the plan's first phase (the signing key, the
  receipt's place in the model, whether the action is a component, and the
  command names), each with its options, their outcomes and a recommendation.

In ParleyInn's domain model the pages' "sandbox" is an ephemeral node,
and their "receipt" is the result receipt, a third receipt kind beside head and
purge, not a CI attestation: no enrolled CI key signs it.

## What was decided

On 28 November the stakeholder chose option 1 of the design, one sandbox per
thread, and set its form: ParleyInn provides the check program as subcommands
of the one `parinn` binary and the verification as a GitHub action shipped with
each release. This repository will use both from an earlier release locked in
`mise.lock`, and other projects take them over the same way.

The same day the stakeholder took the recommended answer to each question on
the third page:

- The editing node's own key signs: its device key, or on an ephemeral node the
  token key its access token carries, with its token certificate.
- The receipt is a result receipt. Its results keep evidence class `own check`.
- The GitHub action is a way the CLI ships, and the way another project adopts
  ParleyInn; it gets a requirement as a stable interface. The check script is a
  template each project owns, until the launcher takes over.
- The commands are `parinn result-receipt plan|make|check`.

[Plan 2611280904](../../../plan/2611280904_review-receipts/plan.md) carries
that out.

## Limits

Nothing on any of the pages was built. The figures are the pages' own: measured in
one web sandbox and from GitHub's Actions API on 26 to 28 November, or
estimated from those parts, and each is labelled. The ruling on ENG-21 that
the design asks for is not made yet.
