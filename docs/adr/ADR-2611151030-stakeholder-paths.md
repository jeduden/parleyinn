---
id: ADR-2611151030
title: "The stakeholder reviews only what guides the review"
status: proposed
summary: >-
  CODEOWNERS names the stakeholder only on what guides the review: the
  agent instructions, the review's crates with their cases, CI and the
  merge rules, the manifests, tool pins, bans and acceptances CI works
  from, and the linter configuration. Every other pull request,
  the specification and the scenarios among them, merges on the review
  gate's approval alone, opened ready with auto-merge on. What lost its
  code owner reviews deep. It builds on ADR-2610262245.
---
# ADR-2611151030: The stakeholder reviews only what guides the review

## Context

ENG-21 had the stakeholder approve every change to the specification,
the decision records, the scenarios, the build configuration, CI and
the agent instructions. A requirement lands with its scenario, so
almost every pull request touched `docs/srs/` or `features/`. Each
then waited twice for the stakeholder: for a comment naming the
changed paths, and for an approval. Agents also opened pull requests
as drafts, which the review skips, and asked the stakeholder to mark
them ready. On 15 November 2026 nine open pull requests were drafts.

The stakeholder ruled that day: no pull request waits for a human
review unless it changes the code or text that guides the automated
review.

## Decision

- CODEOWNERS names the stakeholder on what guides the review, and on
  nothing else:
  - the agent instructions the review's agents load, and the sections
    they include: `CLAUDE.md`, `AGENTS.md`, `.claude/`, `docs/agents/`;
  - the review's code, cases and scorecard: the `tooling/review-*`
    crates;
  - CI and the merge rules: `.github/`, which holds the workflows, the
    rulesets and CODEOWNERS itself, and `scripts/`;
  - the manifests and tool pins CI builds and checks with: the
    `Cargo.toml` of the workspace, of each crate and of the fuzz crate,
    whose `[lints]` decide whether the bans take effect,
    and the mise files and each `rust-toolchain.toml`, which pin the
    tools
    the gates run;
  - the bans and acceptances CI holds the code to: every
    `clippy.toml`, `deny.toml`, `codecov.yml`,
    `tooling/benchmark/accepted.txt` and `tooling/eval/recall-v1.txt`;
  - the linter configuration, `.mdsmith.yml`.
- The specification, the decision records, the scenarios, the size
  list and Rust source outside the review's crates have no code owner. A pull
  request that touches none of the stakeholder's paths merges on the
  review gate's approval and green `CI`.
- The review bundle keeps the review class deep for the paths that
  lost their code owner, so the agents weigh them as before.
- An agent opens its pull request ready, never as a draft, and
  enables auto-merge. GitHub merges it once `CI` and the `review-gate`
  check pass. An agent asks the stakeholder for nothing on such a pull
  request; the 5 US dollar cap of ENG-28 bounds what its reviews spend.
- Main's ruleset does not change. It still requires a code owner's
  approval where CODEOWNERS names one, and the bundle still blocks a
  change to such a path until a stakeholder comment names it.
- ENG-29 does not change, and the review bundle is what enforces it
  while its scenario is `@pending`: a change to an invariant, or to a
  row ENG-29 marks for I2 review (PRV-10 to PRV-13 today), is a
  blocking deterministic finding until an accepted record in the same
  change names ENG-29 and each id, and the stakeholder commented on the
  pull request. ENG-29 also has that record hold a named human security
  reviewer's approval. The bundle does not read the approval; the
  review's agents and the stakeholder, whose comment the finding waits
  for, do. The workflow drops every comment an agent posts, so no
  agent can write that comment, and the bundle is the stakeholder's.
- The bundle raises the same finding for a change to ENG-29's own row.
  That row holds the list of marked rows, and the specification no
  longer has a code owner: without the check, one change approved by
  agents alone could take PRV-10 off the list, and a second could then
  change PRV-10. ENG-29's text does not say this yet. Saying it there
  is a change to ENG-29, which needs its own accepted record naming a
  human security reviewer, so it is left to a change of its own.
- The size list (`tooling/engineering/oversize.txt`) loses its code
  owner and keeps its ratchet. A listed file's lines must equal the
  file's, so most changes to a listed file edit the size list, and an
  owner there would make each of them wait. ENG-34 instead has the
  review bundle block a change that adds a file to the size list or
  raises a file's listed lines until a stakeholder comment names the
  size list. A change that only lowers or removes lines waits for no
  one.

## Alternatives

- **The review's own text and code alone.** CI would lose its code
  owner. `CI` runs as the pull request defines it, so a pull request
  could empty the checks it must pass, with only the agents to notice.
- **Keep the specification too.** The reviewers check a change against
  the SRS, so its text steers them. But nearly every pull request
  changes a row or a scenario, so nearly every one would still wait.
  The bundle's ratchet already blocks a live scenario deleted or put
  back on `@pending`, and ENG-29 guards the invariants.
- **Review drafts.** Agents would need no rule, but a draft is how an
  author says the work is not ready, and each review costs money.
- **A workflow that merges.** The reviewer app would need to write to
  main, and ENG-28 keeps it an identity that authors no change.

## Consequences

- A wrong change to the specification or a scenario can now land on
  the agents' approval alone. The deep class, the ratchet and ENG-29
  are what stand against it, and the stakeholder reads main after the
  fact.
- A pull request that mixes a stakeholder path with other work still
  waits as a whole, so agents keep such a change in its own pull
  request.
- Rust source now has a code owner in the review's crates, which
  ENG-21 forbade before.
- The acceptance files stay the stakeholder's, as ADR-2610142300 and
  ADR-2610262245 rest on. The size list has no code owner, yet only
  the stakeholder's comment lets it grow (ENG-34), so "the size list
  only shrinks" keeps a guard no agent can lift. `SECURITY.md` and
  `DEPENDENCIES.md` change on the agents' approval alone.
- A pull request that adds a crate or a dependency changes a manifest,
  so it still waits for the stakeholder.
- The domain model calls a new concept a stakeholder decision
  (`docs/domain-model/index.md`). No code owner holds the model, so
  the domain-model agents' blocking findings are what enforce that.
- Auto-merge is the author's act. A pull request whose author forgets
  it is approved and waits for someone to merge it.
- The pull request that lands this decision touches the stakeholder's
  paths, so it waits for the stakeholder as before.

## Approval

Proposed. The stakeholder's approval of the pull request that lands
this record accepts it.
