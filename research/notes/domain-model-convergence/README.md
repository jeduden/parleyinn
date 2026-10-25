# Domain-model convergence

The work that makes ParleyInn's domain model and SRS ready to build at
milestones M1 to M5. Rounds 0 to 7 ran a review loop that did not
converge; [process-evaluation.md](process-evaluation.md) gives its numbers
and [ledger-patterns.md](ledger-patterns.md) the patterns behind them.
From 9 October 2026 a fixed sequence of steps replaces that loop.

## What the steps answer

| Pattern in the ledger                               | Answered by        |
| --------------------------------------------------- | ------------------ |
| One rule fixed case by case, in nine chains         | Chain passes       |
| A lifecycle event crossed with derived state        | The lifecycle grid |
| Fixes add text that the next review finds faults in | Fix rules          |
| The model repeats the SRS, then lags it             | Deduplication      |
| Findings follow where the review looks              | Area reviewers     |
| The verifier refutes nothing; labels mislead        | Adversarial bar    |
| Deferred questions inside M1–M5                     | Step 1             |

## Rules that hold throughout

- **Scope.** M1 to M5, as [m1-m5-scope.md](m1-m5-scope.md) sets it. A
  finding that arises only from M6 to M9 features is deferred to OQ-47
  with its milestone.
- **Sources.** A question is settled only by the invariants, the SRS, the
  model or a recorded stakeholder decision. ADRs are history, not a
  source. `docs/srs/invariants.md` never changes; a finding that would
  need it goes to the stakeholder.
- **One ledger.** Every theme gets one entry in
  `plan/2610012322_parinn-for-agent-fleets/finding-ledger.json`, fixed,
  deferred or accepted, citing the sentences that close it.
  `TestFindingLedgerIsCarried` fails when one of them leaves its file.
  When a step moves a cited sentence, it moves the citation with it.
- **Rules live in the SRS.** The model keeps concepts and relations and
  cites the requirement that holds each rule. A fix states its rule once,
  in the SRS, at the level of its chain or grid row, never as one more
  case. A model edit is a concept sentence or a citation.
- **The model does not grow.** No step leaves the model longer than it
  found it.
- **Recommendations decide.** A design question whose verdict carries a
  safety-first recommendation is settled by it, and the step records it
  as the stakeholder's decision (stakeholder, 9 October 2026). Only a
  question with no clear recommendation, or one that would touch an
  invariant, goes to the stakeholder.
- **Each decision is swept.** When the stakeholder decides a design
  question, its consequences are walked through the lifecycle grid. The
  edits for every cell it touches land with the decision, not in a later
  review.
- **No regression pass and no second review.**

## Steps

1. **Settle the deferred questions inside M1–M5.** OQ-43 (three themes,
   M1, spike S4) and OQ-49 (M5) go to the stakeholder with their recorded
   options. A part that needs spike S4's evidence keeps its interim rule,
   which the stakeholder confirms for M1.
2. **Lifecycle grid.** One SRS table, in a new section, so no file at its
   token budget grows. Its rows are the lifecycle events that ship in
   M1–M5: node identity change, node clone, backup restore, harness
   resume, subagent start, run end, run-seat key loss, quarantine and
   release, purge and retention, configuration acceptance, joining and
   leaving one's own rooms, and ingest. Its columns are the derived
   artifacts I10 lists, plus provenance and room membership. Each cell
   names the requirement that decides it, or says "no effect". A cell no
   requirement decides is filled by a tightening edit, or goes to the
   stakeholder when it is a real choice.
3. **Chain passes.** There is one pass per open chain: backup restore and
   clone, pin version, recall taint, configuration acceptance and
   quarantine reach, plus the run-seat key once OQ-43 is settled. Each
   pass writes the chain's general rule once in the SRS. The case clauses
   that restate the rule are replaced with a citation of it, and the pass
   is checked against the chain's rows of the grid.
4. **Deduplication.** Rule text the SRS states leaves the model, and the
   requirement id stays. The model ends at or below 14,826 words, its
   size when the ledger began.
5. **One final review, then stop.** It runs as the next section says.
   Its blocking themes are fixed, and every other theme is deferred.

## The final review

Four reviewers each take one area, reading the model, the requirements
it cites and the grid:

- the record and the node's lifecycle;
- trust and the closed paths;
- pins and restore;
- configuration, seats and keys.

[review-brief.md](review-brief.md) and
[final-review-workflow.js](final-review-workflow.js) were rewritten for
this review; the workflows of rounds 0 to 7 are in the history. A
verifier then judges each theme against the bar, and its default is "not
blocking". The reviewers' own type labels count for nothing.

- A **blocking** theme breaks an invariant in M1–M5 behaviour, or sets
  two normative statements against each other. The verifier quotes the
  broken invariant clause, or the two statements.
- A blocking theme is fixed at its rule, and its requirement's pending
  scenario gains a step for it.
- Any other theme is deferred to one open question of known gaps that
  M1's implementation settles.

## Done

- Steps 1 to 5 are complete. The final review found at most five
  blocking themes, and each is fixed.
- The ledger check, `mdsmith check .`, `go test ./...` and the drift
  suite pass.

The work stops for a report to the stakeholder in three cases:

- the final review finds more than five blocking themes;
- a step leaves the model longer than it found it;
- a step needs an invariant to change.

Each step records in its note the model's word count, the themes it
opened and closed, and how many of them sit on text the work itself
added. These are the figures that showed the old loop failing.

## Rounds 0 to 7

The first process ran blind review rounds, a regression pass after each
round, and the rule "done after two clean rounds". Each round's
findings, matches and verdicts are kept in `round-<n>/`.
