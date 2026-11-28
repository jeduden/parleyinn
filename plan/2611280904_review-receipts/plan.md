---
id: 2611280904
title: "Checks and review on the editing node, verified on GitHub"
status: "🔲"
summary: >-
  The editing node runs the default branch's checks and reviewers on
  each commit and signs a result receipt. A GitHub action, code only,
  verifies it, and the forge runner runs only what it does not answer.
  The default branch's checks and review run again after every merge.
  ParleyInn provides the subcommands and the action, and this
  repository will use them from an earlier release locked in mise.lock.
model: opus
depends-on: []
---
# Checks and review on the editing node, verified on GitHub

## Goal

A change reaches a green check on GitHub minutes sooner. Its checks
and its review run once, on the node where the agent edits. GitHub
verifies a signed record of them and does not run them again. The
quality bar stays as it is: the default branch's checks, reviewers and
decision rule, and both run again after the merge.

## Context

The research and the design are kept in one research note:

- [the note](../../research/notes/review-attestation/README.md), with
  what was decided;
- [the status quo](../../research/notes/review-attestation/status-quo.html);
- [the design](../../research/notes/review-attestation/design-options.html).

On 28 November the stakeholder chose the design's option 1. The
node that edits, an ephemeral node in a cloud environment, also checks,
reviews and signs; no second node is involved. Three conditions came
with it:

- ParleyInn uses this itself, on this repository.
- ParleyInn provides it: subcommands of the one `parinn` binary, and a
  GitHub action released with it.
- This repository depends on an earlier release of both, locked in
  `mise.lock`, so a pull request never changes the code that judges
  it.

The design measured the parts of a round. On the forge runner, CI and
the review took 9 min 51 s. On an ephemeral node the checks took 4 min 25 s on a
fresh build and 2 min 36 s again. Main's CI after a merge took a median
of 7 min 43 s. From these parts the design estimates a change of three
rounds at about 37 minutes today. It puts the same change at about 23
minutes with receipts, and about 18 once every check declares its
inputs.

What was searched, and what is reused:

- [review-run](../../tooling/review-run/src/key.rs) already keys each
  reviewer's answer by everything it is a function of, keeps answers in
  a store and takes a stored answer in place of a run. The receipt's
  review part is that store with its kept outcome. The tool stays this
  repository's and plugs into `parinn` through a narrow interface. The
  receipt carries the review outcome; it states no verdict of its own.
- [scripts/ci-gates.txt](../../scripts/ci-gates.txt) and
  [scripts/ci-gate.sh](../../scripts/ci-gate.sh) already name every
  check CI runs and run each one. That list is engineering tooling, not
  a model concept. It gains each check's inputs and class.
- The [domain model](../../docs/domain-model/index.md) already has a
  receipt, the `own check` evidence class, a CI key and a CI
  attestation. It has no result receipt, names no key that would sign
  one, and no B0 component starts a check.
- The Ed25519 and SHA-2 crates already have accepted decision records,
  so no new dependency is planned.
- The signed release pipeline (ENG-20) already publishes `parinn`. No
  release has shipped yet, so the first release is part of this plan.

Three constraints shape the cut:

- The core starts no program and opens no socket (I4, ENG-16). So in
  the first cut `parinn` works out the checks, keeps answers, signs and
  checks receipts. A check script, a template each project copies and
  owns, starts one check per tool call of the editing run. The launcher
  takes the start over only after ENG-29's decision record with a named
  human security reviewer.
- ENG-21 asks for "a reviewer other than its author". Letting the
  pull-request author's own result receipt carry the review outcome needs
  the stakeholder's ruling, written into ENG-28 and a decision record,
  before phase 7.
- ParleyInn posts no forge approval. The action reports; this
  repository's own workflow posts `review-gate`, as today.

On 28 November the stakeholder also took the recommended answer to
each open question of phase 1, as the
[options page](../../research/notes/review-attestation/open-questions.html)
sets them out:

- The editing node's own key signs: its device key, or on an ephemeral
  node the token key its access token carries, with the token
  certificate in the receipt. SEC-10 already allows both. A fixed
  domain tag opens the signed bytes, so a receipt never reads as a
  principal act. The default branch lists device keys, never one key
  per node, in a file on a path `CODEOWNERS` names, so no pull request
  adds a key without the stakeholder.
- The concept is the result receipt, a third receipt kind beside head
  and purge. It states the own-check results its node recorded for one
  commit, the digest of the `parinn` release and the review outcome.
  Its results keep evidence class `own check`; LANE-05 does not change.
  To stay own checks, each check runs as a tool call of the editing
  run, so its exit status is a capture. The receipt binds each result
  to its entry in the default branch's list, so a substituted command
  never counts.
- The GitHub action is a way the core's CLI ships to a forge, a third
  shipping form under OQ-32, and the way another project adopts
  ParleyInn. It gets a requirement as a stable interface. The check
  script ships as a template each project owns, until the launcher
  takes over; a requirement settles SEC-29 and ENG-16 for it before it
  ships.
- The commands are `parinn result-receipt plan|make|check`, beside
  `head-receipt make|check`. The node-wide `verify` keeps checking the
  node's own record.

Not taken now: a second ephemeral node that checks and signs, hosted
harness sessions that run the reviewers, and a second OS user on a
machine of the stakeholder's that holds the key. The design keeps them
as option 2 and variants.

## Tasks

1. Specify: the result receipt in the domain model with the key that
   signs it, the requirements with their scenarios `@pending`, the
   subcommand names, and the decision record (phase 1).
2. Trials on an ephemeral node, before code: a pushed branch that
   carries result receipts and nothing else, the `claude` version CI
   locks under the node's model access, the checks as an unprivileged
   user in a process tree of their own, and checks and review side by
   side on four cores (phase 2).
3. `parinn result-receipt plan` works out the checks for a commit
   from the default branch's list and starts none of them. Each check
   runs as a tool call of the editing run, and `parinn result-receipt
   make` signs the captured results (phase 3).
4. `parinn result-receipt check` checks a result receipt with no
   network, and a GitHub action released with `parinn` runs it as a
   stable interface other projects rely on (phase 4).
5. `review-run` runs the default branch's reviewer definitions with the
   locked CLI and writes their answers and review outcome into the result
   receipt; the `check-loop` skill replaces the line in CLAUDE.md that
   asks for `review-local.sh` before a push (phase 5).
6. First release, locked in `mise.lock`. The verification job reports
   beside CI and decides nothing. After every merge the default
   branch's review runs again on the forge runner (phase 6).
7. The result receipt is accepted as the check result on the paths no
   code owner holds, after the ENG-21 ruling: the file on the default
   branch that says which keys may answer what and which paths the
   forge runner takes first, the stop when the default branch fails,
   the spot re-run and the planted defects (phase 7).
8. Every check declares what it reads, check by check (phase 8).
9. A second project adopts the locked release and the action
   (phase 9).

## Execution

| Phase | Model  | Gate                                                                                             |
| ----- | ------ | ------------------------------------------------------------------------------------------------ |
| 1     | opus   | The SRS gate test passes with each new scenario `@pending`; the domain-model agent finds nothing |
| 2     | opus   | Each trial's outcome is written in its handoff with the command that showed it                   |
| 3     | opus   | `parinn` in an isolated `PARINN_HOME` signs the check script's exit statuses for a commit        |
| 4     | opus   | A result receipt with one byte changed fails verification; a drift case proves the check fails   |
| 5     | opus   | The receipt's answer keys match those `review-run` computes on the forge runner                  |
| 6     | sonnet | The verification job lists each difference from the forge runner on ten pull requests            |
| 7     | opus   | An ordinary pull request merges on its receipt; a forged line is refused                         |
| 8     | sonnet | A change to a declared input changes its digest; a drift case proves it                          |
| 9     | opus   | The second project's pull request merges on a receipt verified by the pinned action              |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                                              |
| --- | ------ | ---------------------------------------------------------------------------------- |
| 1   | 🔲     | [Specify the result receipt, its requirements and its decision record](phase-1.md) |
<?/catalog?>

## Acceptance Criteria

- [ ] A commit on the editing node gets a result receipt signed by the
      locked `parinn`, with no push and no workflow run
- [ ] The GitHub action verifies a result receipt with no agent, no
      model credential and no network, and refuses one changed byte
- [ ] A pull request on an ordinary path gets `review-gate` about half
      a minute after its push once every check declares its inputs
- [ ] Every path a code owner holds still gets every check and every
      reviewer on the forge runner before the merge
- [ ] The default branch's checks and review run after every merge,
      and a check that contradicts a receipt stops receipts until the
      default branch passes again
- [ ] This repository runs `parinn` and the action from an earlier
      release locked in `mise.lock`, never from the change under review
- [ ] A second project uses the same release and action
- [ ] `mdsmith check .` passes and CI is green
