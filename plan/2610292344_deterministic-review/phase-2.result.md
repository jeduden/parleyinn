---
n: 2
title: "The review workflow runs review-run"
status: "✅"
result: true
summary: >-
  review.yml runs review-run in place of the orchestrating agent, and
  ENG-28, its scenario, binding and drift cases now ask for a review run
  by code; the gate names an unanswered agent without an orchestrator,
  ADR-2610300031 records the decision, since accepted on the
  stakeholder's consent, and the generalists keep their models.
---
# Phase 2 handoff

## Done

- `review.yml`'s review job installs the pinned Claude Code command
  line (`CLAUDE_CODE_VERSION` 2.1.312, npm, as the evaluate job does).
  It builds `review-run` with no secret in the environment, then runs
  `cargo run --locked --quiet --offline -p review-run -- --workdir .
  --parallel 32` with only `CLAUDE_CODE_OAUTH_TOKEN` added. The job's
  `outcome` and `consulted` outputs read `review/outcome.json` and
  `review/consulted.json`, and the cost step reads `review/usage.json`.
  The `claude-code-action` step, the class's turn budget and the run
  log's jq parsing are gone. Permissions stay `contents: read` and
  `pull-requests: read`, the job holds no reviewer-app key, and the
  agents now get no GitHub token at all. zizmor is clean.
- ENG-28's row asks for the review to be run by code, not by an
  orchestrating agent. A tool runs every dispatched specialist, then
  every dispatched generalist, and writes the outcome from what they
  answered. A dispatched agent that did not answer fails the review.
  The ENG-28 scenario holds the agents' job to `review-run` and refuses
  an agent session that would route the review. It holds review-run's
  tool lists and every `--allowedTools` an agent job passes to
  `review/reports`. New drift case: the review routed by
  `claude-code-action`. The cases that inject the key and a write tool
  now target the Review step.
- The gate refuses with `review: dispatched but not consulted: …`.
- PR #98 (`claude/wizardly-faraday-u3k1e6`) merged cleanly. Its
  generalist report shape starts a dropped line with `dropped —`, which
  review-run already leaves out.
- ADR-2610300031, written `proposed` with its Approval section left
  for the stakeholder. The stakeholder has since consented in PR #98's
  comment 6534291701, which its Approval cites: it is `accepted`, and
  ADR-2610262245 is `superseded` by it.
- `docs/development.md` and `docs/review-evaluation.md` describe
  review-run.

## Decisions

- `--parallel 32`, as ADR-2610300031 says: a dispatch of up to 32
  specialists fits one wave. The job's 120-minute cap fits the
  three-wave worst case of 90 minutes and the setup.
- The agents' job is found by `-p review-run`, and also by
  `claude-code-action@`, so an agent session is still held to the
  agents' rules wherever it appears.
- ADR-2610262245 is marked superseded only once ADR-2610300031 is
  accepted, since a proposal changes no decision (ENG-26).

## Next

- Phase 3 retires the orchestrator: `.claude/skills/review`, its ENG-32
  component and cases, the runner's orchestrator prompt and schema,
  and every remaining mention. That includes the generalists' own text
  ("the review's orchestrator calls you", which changes reviewer
  components and needs their cases run), the dispatch module's doc
  comment, ENG-32's row and scenario, and `review-evaluation.md`.
- The gate's live check, one review decided on its first run, needs
  this change on main.
- A dispatch of more than 32 specialists takes another wave, which
  the 120-minute cap still holds.
