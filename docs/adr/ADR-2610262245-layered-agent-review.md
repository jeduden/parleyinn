---
id: ADR-2610262245
title: "Layered agent review with a required review-gate check"
status: superseded
superseded-by: ADR-2610300031
summary: >-
  One orchestrating agent consults the specialists and generalists the
  review bundle dispatches, the deep review weighs on Opus, and the
  gate's decision becomes a check main requires from the reviewer app
  alone. Supersedes ADR-2609301941.
---
# ADR-2610262245: Layered agent review

## Context

ADR-2609301941 has one agent review each pull request with read,
glob and grep, and a token that writes reviews and reads checks. ENG-21
now leaves Rust source without a code owner, so the agent review is
all that stands between a change to the gates or the security crates
and main. One agent reading every file at once misses what a reader
of one language or one section catches. And any other reviewer's
approval satisfied main's ruleset as well as the agent's did.

## Decision

- **Layers.** `review-bundle` dispatches, from the changed paths and
  the rows the change edits or code names, a low-effort specialist per
  language, SRS section, model file and test layer, each under the
  medium-effort generalist that weighs its findings. A deep review
  consults each generalist's deep instance: the same template, on Opus
  at high effort with 40 turns.
- **Routing.** One orchestrating agent, on Sonnet at low effort, calls
  the specialists in batches of at most 20, then the generalists. It
  gains the Agent tool for this, and routes by path, never by text.
  Each specialist reads its own diff, the bundle's cut of the change
  to the files that brought it, or the whole diff when a row alone
  did; the generalists read the whole diff.
- **Reports as files.** Each specialist writes its report to
  `review/reports/<its name>.md`, the one path any agent may write;
  the generalists read the files and write nothing. The post job's
  gate fails a review whose specialist's file is missing or empty, or
  does not end with the `outcome:` line that agent answered in the run
  log. A specialist answers with that line and the file's path alone,
  so a report it rewords before answering still passes; the check
  cannot see a body changed with the outcome line kept, since every
  agent of the run holds the same write rule.
- **The gate checks the routing.** `review-gate` fails without posting
  when a dispatched agent was not consulted, as the orchestrator's run
  log shows.
- **A required check.** The post job's token may also write checks.
  The gate posts its decision as the `review-gate` check on the
  reviewed head, passing only on an approval, and main's ruleset
  requires that check from the reviewer app alone, by its app id.
- **What stays.** The `workflow_run` trigger, main's definitions, the
  read-only agent job, the key held by the post job alone and the
  gate's decision table stay as ADR-2609301941 has them.

## Alternatives

- **Keep one agent, on Opus.** Cheapest to run, but the first layered
  run on this decision's own pull request found two broken accepted
  decisions and a language gap no single reading had found.
- **Code owners for the agent.** GitHub takes only users and teams as
  code owners, so an app cannot be one.
- **Per-call model overrides instead of deep instances.** The
  orchestrator could pass `model: opus`, but only front matter sets a
  turn budget, and the gate could not check an override it never sees.

## Consequences

- ADR-2609301941 is superseded, naming
  this record.
- ADR-2610142300 rests on its acceptance file being the
  stakeholder's. CODEOWNERS keeps `tooling/benchmark/accepted.txt`
  and `tooling/eval/recall-v1.txt` owned for that reason, now that
  `tooling/` is not.
- A deep review spends about 1.5 million model tokens and ten minutes;
  the review job's cap is 45 minutes.
- An agent the dispatch names must be defined on main first, since
  the review runs as main defines it.
