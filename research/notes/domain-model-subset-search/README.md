# Domain-model subset search

The search for the largest subset of ParleyInn's domain model that is
consistent, secure and useful. A candidate subset is assembled from the
model by dropping whole features, reviewed blind, scored, and repaired.
The search stops when a candidate has no findings and adding any
excluded feature brings findings back.

Status: in progress, selection only. The stakeholder asked for it during
the blind domain-model reviews of the parinn-for-agent-fleets plan, and then
ruled that the search selects feature combinations of the current model and
never changes its text. `annotated/` is the original model with feature
spans; `check_annotation.py` (run without `--no-orig`) proves it equals
`docs/domain-model/` and `docs/srs/invariants.md` byte for byte. Nothing
here is normative.

Versions v1 to v6 (`annotated-v1/` … `annotated-v6/`, the repair rulings and
`invariant-edits.md`) record an earlier phase that also repaired the text.
They are kept as history; the selection search does not use them.

## Method

1. **Genes.** `features.json` splits the model into a core and 47
   optional features. Each has its entries (concept items), the
   features it requires, soft links, a value and a risk. `items.json`
   and `inventory.json` list every list item of the model.
   `coverage.py` checks that each item belongs to exactly one feature.
2. **Annotated sources.** `annotated/` holds the ten concept files, the
   hub and the invariants. Spans `⟦f:ID⟧ … ⟦/f⟧` mark each feature's
   text and nest where a clause needs two features. The files end in
   `.md.ann` so that mdsmith leaves the markers alone.
   `annotate-brief.md` gives the marker rules, and `check_annotation.py`
   checks balance, ids and dangling terms.
3. **Assembly.** `assemble.py <genome> <outdir>` keeps the core and the
   genome's features, drops every other span and regenerates the hub's
   catalog. It is deterministic. Its closure check reports a term
   defined only in a dropped feature that is still used.
4. **Evaluation.** Blind reviewers read only the assembled
   `candidate.md` through one lens: consistency, security or
   usefulness (`eval-brief.md`, `eval-workflow.js`). Each finding
   quotes the candidate. `attribute.py` maps the quote back to the
   innermost feature span, so a finding blames one feature or the core.
5. **Fitness.** `ga.py` keeps the ledger (`ledger.json`) and scores a
   candidate as value − 4 × needs-fix findings − 0.5 × dangling terms.
   It also does closure, blame, mutation and crossover.
6. **Selection GA.** `ga_round.py` fits each feature's needs-fix rate per
   review from the ledger (`--only v0`: reviews of the original text), then
   breeds genomes by crossover and mutation, closed under `requires`. Each
   pick is a Thompson sample: every rate is drawn from its posterior, so
   rarely reviewed features get explored and well-measured ones settle.
   `ga-select.js` runs one generation as a workflow: breed, one blind probe
   review per candidate, record and attribute the findings, commit. The
   best candidates get a strong review to confirm.

## Results

The stakeholder stopped the search on 8 October 2026. Strong reviews ran
three blind reviewers, one per lens; probes ran one reviewer covering all
three lenses on a faster model, which finds fewer issues and confirms
nothing. The repair phase (v1 to v6) is history; these are the
selection results on the original text.

| Candidate                       | Features | Needs fix | On core | On features |
| ------------------------------- | -------- | --------- | ------- | ----------- |
| v0-all-r2 (all features)        | 47       | 9         | 5       | 4           |
| v0-ga2-4 plus browser-room-view | 46       | 6         | 2       | 4           |
| v0-ga2-4 plus store-protection  | 46       | 9         | 2       | 7           |
| v0-ga2-4                        | 45       | 8         | 4       | 4           |
| v0-ga1-5                        | 39       | 11        | 3       | 8           |

- **Selection buys nothing measurable.** Every combination reviewed
  strongly, from 39 to 47 features, lands at 6 to 11 needs-fix findings,
  about the noise of one strong review. The full model sits in the middle,
  so the largest subset with the fewest findings is, within that noise,
  the whole model.
- **What selection cannot remove.** About five needs-fix findings sit in
  core text, which every combination includes. The rest spread thinly
  over features; principal-keys is the one blamed in most reviews, for
  ownership, admission and bars that rest on principal keys nothing
  certifies before PRV-10.
- **Reviewer attention saturates.** A review reports a roughly fixed
  number of findings and spreads them over the text in front of it: the
  core alone drew 21 core findings, the same core inside 39 features 3.
  Totals compare candidates of similar size only.
- **The first-round reviews are stale.** g0 (22 needs-fix for all
  features, 4 on browser-room-view) used an earlier eval brief, before the
  settled decisions were added; the ledger marks them `stale` and
  `ga_round.py` skips them. The GA's early preference for leaving out
  browser-room-view came from them.
- **Two checks, stopped after two of three lenses.** Counting only the
  consistency and security lenses, a second review of all 47 features
  (`v0-all-r3-partial`) found 11 needs-fix findings where the first
  (`v0-all-r2`) found 7: the noise of one strong review on one
  candidate. All features but principal-keys and its ten dependents
  (`v0-nopk-partial`, 36 features) found 12, so dropping that cluster does
  not lower the findings either.

## Running it

From this folder, with Python 3 and nothing else:

```sh
python3 check_annotation.py                     # text unchanged, spans balance
python3 assemble.py genomes/g0-core.json out/g0-core
python3 ga.py rank                              # ledger by fitness
python3 ga_round.py v0-ga1 6 --only v0 --seed 1   # breed one generation
```

`out/` is derived and ignored by git. Rebuild any candidate from its
genome with the matching snapshot. Copy `annotated-vN/` over
`annotated/` to rebuild a candidate of an earlier generation exactly as
its reviewers read it. `eval-workflow.js` is a Claude Code workflow
script. It takes `{"ga": "<absolute path of this folder>", "jobs":
[{"cand", "lens", "model"?}]}`. `collect_wf.py` files its output into
`evals/` and the ledger.
