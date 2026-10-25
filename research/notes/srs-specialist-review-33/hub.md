I found three real defects at HEAD. The first is drift between a hub relation and the SRS. The second is a gap the hub's recall relation leaves open. The third is a scenario defect that belongs to the trust-and-flow specialist. Everything else in my part of the model checked clean.

**What checked clean**

- **Catalog** (`/home/user/cairn/docs/domain-model/index.md`): all ten concept files are listed. Each title, order and summary matches the file's front matter, and each concept file has its specialist agent under `/home/user/cairn/.claude/agents/domain-model-*.md`.
- **Excluded terms**: I searched every excluded term across `docs/srs/*.md` and `features/*.feature`. Every hit is an allowed use:
  - "harness session", `SessionStart`, `session_id`, `allow-session`
  - `~/.claude/projects`, `--scope project`, and ParleyInn's own software project (OQ-09, ENG-25)
  - the `operator` provenance class, plus FTS5 and query "operators", which are no role and no person
  - LANE ids and file names
  - "read-only" as an adjective, and "hiding" in REC-17
  - the SRS change log, and ENG-28's `verdict` field and "review outcome"
- **Concept files**: none uses an excluded term outside its allowance.
- **Names**: every MCP tool in §9.2, every CLI command and option in §9.5, and every settings key in §9.6 follows the hub's rules. That covers `room_<act>` and `room_create`, `<concept>_<verb>`, the one-word utilities, kebab and snake case, and `<concept>.<room>.<qualifier>`. Every `parinn …` command a scenario or other SRS section uses exists in §9.5. Every MCP tool a scenario uses exists in §9.2.
- **Relations**: the ids they cite (PIN-10, LANE-01, LANE-16, LANE-29, SEC-27, OQ-41, RCL-05) exist. Apart from finding 1, they agree with the concept files and the requirements. No two relations contradict.

**Findings**

1. **LANE-01 lacks the ingest-seat rule in hub relation 6** (needs-fix, SRS).

  - `/home/user/cairn/docs/srs/05b-lane-requirements.md:14` routes every run event to "its run seat in the room it works in … else to its personal-room seat, never refused".
  - Hub relation 6 (`index.md:70-72`) says what `parinn ingest` appends for a witnessed run "goes instead to the seat ingest starts beside that run seat". REC-19 (`/home/user/cairn/docs/srs/05-functional-requirements.md:34`) requires the same, including "beside a personal-room seat".
  - The hub (relation 2), Member and Seat keep that seat apart from "its personal-room seat". So for ingested appends LANE-01 contradicts REC-19 and the relation.
  - Smallest fix: in LANE-01, after "else to its personal-room seat, never refused", add "; what `parinn ingest` appends for a witnessed run goes instead to the seat REC-19 starts beside that run seat".

2. **Default recall versus a room that has turned foreign** (question for the stakeholder).

  - RCL-05 (`05-functional-requirements.md:82`) defaults recall to "the calling agent's current run, through every writer of its seats".
  - RCL-10 (`:87`) says "Recall MUST reach a foreign room only through a `room` parameter naming it in that call".
  - A run whose seat sits in a room every seat of its principal has since left, or lost to a kick or a bar, has events in a foreign room's writer. The two requirements then conflict.
  - Hub relation 10 (`index.md:96-98`) covers only extended recall ("extends … never to a foreign room unless the call names it"). Foreign room (`places.md:51-53`) leaves it "outside every extended recall scope", which implies the run scope keeps it.
  - Fix if the ruling follows the model: in RCL-10, change "reach" to "extend recall to", and add "beyond the calling run's own events, which recall marks untrusted". Otherwise, exclude those writers in RCL-05 and in the Foreign room definition. Places and trust-and-flow should see the ruling.

3. **A "counter" for every recall call** (trust-and-flow specialist).

  - `/home/user/cairn/features/recall.feature:94` (RCL-07): "And the counter \"recall_calls\" increases by 1".
  - A Counter (`trust-and-flow.md:138-141`) counts only dropped, rejected, redacted, truncated, coalesced, timed-out or failed operations, as OPS-01 does. It is shown on Health until acknowledged. No requirement names this counter. A count of recalls is a Stat (`stat_list`).
  - Fix: replace the step with "And the count of recalls `stat_list` reports increases by 1".

```json
[
  {"file": "docs/srs/05b-lane-requirements.md", "line": 14, "quote": "else to its personal-room seat, never refused", "term": "seat ingest starts / personal-room seat", "breaks": "Hub relation 6 (index.md:70-72: what `parinn ingest` appends for a witnessed run 'goes instead to the seat ingest starts beside that run seat') and REC-19 (05-functional-requirements.md:34, 'beside a personal-room seat'); hub relation 2, Member and Seat keep that seat apart from 'its personal-room seat', so LANE-01's rule for every event contradicts REC-19 for ingested appends", "fix": "SRS LANE-01: after 'else to its personal-room seat, never refused' add '; what `parinn ingest` appends for a witnessed run goes instead to the seat REC-19 starts beside that run seat'", "kind": "needs-fix", "invariant": false, "hard_to_revert": false},
  {"file": "docs/srs/05-functional-requirements.md", "line": "82-87", "quote": "Recall MUST reach a foreign room only through a `room` parameter naming it in that call", "term": "foreign room / recall scope `run`", "breaks": "RCL-05 defaults recall to the run 'through every writer of its seats', which includes a run seat's writer in a room that has since turned foreign; RCL-10 forbids reaching it without naming it. Hub relation 10 (index.md:96-98) and Foreign room (places.md:51-53) govern only extended recall scopes, so the model implies run scope keeps those writers, marked untrusted", "fix": "Stakeholder ruling. If it follows the model, SRS RCL-10: 'Recall MUST extend to a foreign room only through a `room` parameter naming it in that call, beyond the calling run's own events, which recall marks untrusted'. Otherwise exclude such writers in RCL-05 and in places.md Foreign room", "kind": "question", "invariant": false, "hard_to_revert": false},
  {"file": "features/recall.feature", "line": 94, "quote": "And the counter \"recall_calls\" increases by 1", "term": "counter", "breaks": "Counter (trust-and-flow.md:138-141) and OPS-01 count only dropped, rejected, redacted, truncated, coalesced, timed-out or failed operations, shown on Health until acknowledged; a count of recalls is a Stat read through stat_list, and no requirement names this counter", "fix": "Scenario RCL-07: replace the step with 'And the count of recalls `stat_list` reports increases by 1'", "kind": "other-specialist", "invariant": false, "hard_to_revert": false}
]
```

**Instruction feedback**

- The instructions do not say who owns a contradiction between an SRS requirement and a hub relation when the relation cites a concept (such as "(Run seat)") rather than a requirement id. I treated finding 1 as hub-owned.
- Step 5 checks "identifiers … in the SRS's interfaces and in code" against the hub's rules. The hub's naming rules cover only MCP tools, CLI commands and options, and settings keys. Counter names in scenarios and the example crate names in ENG-02 fall through, and the instructions do not say which specialist reviews them.
- The excluded-terms table limits "session" to harness uses. Only the hub's general sentence about outside things covers a third-party project's proper name cited as evidence ("session-index README", `02-context.md:53`). I read it as allowed; saying so explicitly would help.
- "Search the whole repository" (step 4) conflicts with a caller-scoped review such as this one, which covered only the SRS and the scenarios. A line saying the caller's scope wins would help.
