# Steps of the second process

The record of each step of the [process](README.md). Model words are
`wc -w docs/domain-model/*.md`: 18,193 when the process began, against a
target of 14,826.

| Step | Model words | Opened | Closed | On the work's own text |
| ---- | ----------- | ------ | ------ | ---------------------- |
| 1    | 18,177      | 0      | 2      | 0                      |
| 2    | 18,147      | 0      | 0      | 0                      |
| 3    | 17,419      | 0      | 0      | 0                      |
| 4    | 12,834      | 0      | 0      | 0                      |
| 5    | 12,834      | 29     | 29     | 18                     |

## Step 1: the open questions inside M1–M5

The stakeholder decided on 9 October 2026, and the SRS records each
decision:

- **OQ-43, where a witnessed run's events go while no MCP server holds
  its run-seat key.** The events are held, then ingested. The hook
  handlers append nothing until the server holds the key, leaving an
  ingest marker. A run no MCP server serves reaches the record only
  through `parinn ingest`, and is counted and audited until then
  (REC-19). This closes DM-CD.
- **OQ-43, a lost run-seat key, and a resume with no stable agent id.**
  The interim rules stand for M1, and spike S4 reopens them. DM-D and
  DM-BK stay deferred.
- **OQ-49.** The pin goes to its creating event's room. Where its
  principal has no member seat left there, it goes to the personal
  room, which the confirmation shows (PIN-05). This closes DM-DT.

§5.2 (PRV) moved unchanged to its own file,
`05e-provenance-requirements.md`, so `05-functional-requirements.md`
has room under its token budget for the steps that follow. The ledger's
citations of PRV rows moved with them.

Step 2 sweeps these decisions through the grid's rows for run-seat key
loss, harness resume and ingest.

## Step 2: the lifecycle grid

Six agents read the SRS for 27 M1–M5 lifecycle events, one per row, and
named for each derived artifact the requirement that decides it, or the
gap. The grid itself lands in §8.5. The gaps closed as follows.

The stakeholder decided four questions:

- **Quarantine selectors** stay live for recall, landmark text and
  trusted-only exports. A pin version or landmark recorded after a cut
  quarantine leaves restore only through a widening one. A seat selector
  covers the seat `parinn ingest` starts beside it, never a successor seat
  (SEC-12).
- **Retention** ages events by the node's structural time marks, never by
  a transcript timestamp (REC-15, SEC-22, PRV-01).
- **A copy another node made:** the rooms it brings that the node did not
  hold stay foreign until PRV-10 ships or a verified seal keeps their
  events (ADM-06).
- **A restore fork:** each segment of the copy that differs from the
  node's event at its address is refused (ADM-06, REC-21).

The stakeholder's rule that a recommendation decides settled the rest:

- After a node identity change or a clone, the home's rooms stay the
  principal's and its old seats count as member seats, until PRV-10
  ships (REC-24).
- A purge whose scope holds a pending ingest marker or a run REC-19
  counts is refused until ingest brings them in (ADM-07).
- An act on more than one room goes to the personal room (LANE-01).
- Each subagent's run is its own agent (REC-02).
- A restore keeps the copy's audit log as a read-only archive, and no
  counter derives from it (ADM-06).
- A run's last span closes at its end, recorded as a structural hook
  observation (LMK-01).
- `parinn status` names each setting waiting for acceptance (ADM-11).
- A principal may dismiss a pin candidate by a neutral act (PIN-05).
- A device key or seat key lost while the node identity is unchanged is
  minted anew without counting as a changed identity (REC-24, SEC-10).

Tightenings that follow from the invariants and existing rules landed in
SEC-10, SEC-12, SEC-13, SEC-22, SEC-27, REC-02, REC-13, REC-15, REC-19,
REC-21, REC-24, PIN-05, PIN-10, INJ-05, ADM-04, ADM-06, ADM-07, PRV-01,
PRV-02, PRV-07, PRV-09, LANE-01, LANE-16, LANE-23 and §12.2. The latter
schedules the step-1 decision and SEC-27's lost-key seat in M1. The model
gained the term time mark and the subagent clause; it also dropped
restatements of SEC-12 and ADM-02, so it shrank to 18,147 words.

The last two rows' agents added six recommendations, settled the same
way:

- A run is counted once its hook handlers reach a `Stop`,
  `SubagentStop` or `SessionEnd` before any MCP server holds its key;
  earlier waits are only audited (REC-19).
- The hook handlers tell a nested harness session from their own
  process ancestry, read inside the core (PRV-08).
- A new transcript generation is ingested past the prefix already
  ingested, and a purged line is refused by a keyed hash (REC-07).
- An event a hook's input makes waits in the ingest marker at its
  transcript offset (REC-13).
- `parinn verify` reports a source past its ingest position as pending,
  and fails only for one nothing can resume (ADM-09).
- A hook handler's budget cut is audited under a counter that is not a
  failure counter (REC-13).

The grid's gaps are not review findings, so they open no ledger theme.
The agents also named events beyond the grid's 27 rows: a run's start,
an MCP server that stops for good, a repository identity bound, recall
that returns untrusted content, a hook handler failing open, a
provisional branch identity rebound on push, the lack of an unlink act,
and a locked key store. The final review reads them with the grid.

## Step 3: chain passes

One agent per chain wrote the chain's general rule once, in the row that
governs it, and replaced the case clauses that restated it with a
citation. The ledger's citations moved to the new text.

| Chain                    | Rule in | Themes | Model words |
| ------------------------ | ------- | ------ | ----------- |
| Quarantine reach         | SEC-12  | 5      | −5          |
| Run-seat key             | SEC-27  | 6      | −36         |
| Recall taint             | SEC-13  | 8      | −50         |
| Configuration acceptance | ADM-04  | 8      | −199        |
| Pin version              | PIN-10  | 9      | −264        |
| Backup restore and clone | ADM-06  | 9      | −174        |

Each rule loosens nothing. Three of them tighten what the cases said:

- a quarantine's reach over later matches now covers every path to an
  agent;
- recall taint flows back through a nested session's tool result;
- each flow of SEC-13 is a MUST wherever taint is kept.

The pending SEC-13 scenario gained the step for the new back flow. The
agents noted two points for the final review:

- OWN-10's "every rule level change the principal makes is a principal
  act" sits against ADM-04's rule that a change which only tightens
  needs no acceptance;
- SEC-27's pending scenario has no step yet for a seat key missing from
  where SEC-10 keeps it.

## Step 4: deduplication

One agent per model file took out each rule the SRS states and left the
concept with the requirement id that holds the rule. Each agent kept a
rule no requirement states, and listed it. Where a ledger theme cited a
sentence that left, its citation moved to the SRS sentence that states
the rule: 47 citations moved, and none was dropped.

| File                       | Before | After |
| -------------------------- | ------ | ----- |
| acts-and-roles.md          | 2,247  | 1,596 |
| components-and-surfaces.md | 986    | 678   |
| git-and-forge.md           | 1,206  | 889   |
| harness-facts.md           | 543    | 429   |
| index.md                   | 2,147  | 1,710 |
| pins-and-context.md        | 1,842  | 1,405 |
| places.md                  | 1,214  | 649   |
| principals-and-agents.md   | 1,626  | 1,215 |
| record.md                  | 2,150  | 1,647 |
| seats-and-keys.md          | 1,180  | 939   |
| trust-and-flow.md          | 2,278  | 1,677 |

The model ends at 12,834 words, under the target of 14,826. The SRS did
not change.

Rules the model keeps because no requirement states them, for the final
review to weigh:

- the direction of pin priority, lower first;
- a personal-room seat can neither leave nor be kicked;
- every change of visibility, admission or a room setting is widening;
- a role assignment makes ParleyInn refuse no seat's act;
- the naming rules for budgets, runs, sources and the harness's hooks;
- a natural-language landmark headline never reaches a restore block;
- an event from a peer or a bundle records no recorder.

## Step 5: the final review

Four area reviewers read the model, the requirements it cites and the
grid, following [review-brief.md](review-brief.md); an adversarial
verifier per area judged each theme, defaulting to not blocking
([final-review-workflow.js](final-review-workflow.js)). They reported 34
findings, each a theme of its own. Two pairs are one defect each: a pin
whose text holds a restore delimiter, raised in the trust and pins
areas, and managed policy changing a pin, raised in the pins and
configuration areas. That leaves 32 themes. The ledger records the 29
that are real, and the three refuted ones are named below.

| Verdict   | Themes | Where they went                        |
| --------- | ------ | -------------------------------------- |
| Blocking  | 5      | Fixed at their rule, DM-DV to DM-DZ    |
| Known gap | 17     | OQ-50, for M1, DM-EA to DM-EQ          |
| Later     | 7      | OQ-47 with their milestone, DM-ER on   |
| Not real  | 3      | Refuted by the text the verifier cited |

The five blocking themes, each fixed in the row that governs it, and
each with a step in that row's pending scenario:

- **A pin whose text holds a restore delimiter** (INJ-08 against I3 and
  PIN-08). A pin's text holds none of INJ-08's delimiter sequences
  (PIN-03), so every way a pin version gets its text refuses it.
- **The restore block's limit against the pins it must carry** (INJ-07
  against PIN-08 and I3). The limit bounds all but what PIN-08 and
  PIN-10 make the block carry; a block those alone keep over it is
  audited and counted.
- **Repository redaction patterns** (I1). A repository's `.parinn.toml`
  could add a pattern that erases text that is not secret; only the
  principal's own configuration defines patterns (SEC-08, §9.6).
- **`parinn ingest` after a lost key** (I1 against PRV-11). With the node
  identity unchanged, no key could certify the seat that carries the
  rest of a run; REC-19's personal-room route now covers a missing key
  such a seat needs. The grid's row for a lost key cites REC-19.
- **`parinn uninstall` after a SEC-22 refusal** (ADM-02 against itself).
  Any refusal of the act turning capture off changes nothing.

The verifiers refuted three themes: `kernel_variable_list` names outside
the envelope (RCL-04 already envelopes them), managed policy changing a
pin (raised twice; PIN-01 lets only a principal do it), and a new run
seat's membership after a key loss (LANE-16 and LANE-25 already cover
it).

The last column of the table counts themes that quote a sentence steps
1 to 4 wrote or reworded. It overcounts, since a reworded older rule
counts too. One blocking theme sits on such text: REC-19's route for a
node identity change, written in step 2, which the lost-key fix extends.

The model did not change in this step. The SRS is at 2.31-draft. The
reviewers' findings and the verifiers' verdicts are in
`final-review/verdicts.json`.
