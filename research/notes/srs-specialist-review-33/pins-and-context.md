## Pins-and-context review of the SRS and scenarios at HEAD (round 33, blind)

I found 13 defects in the SRS and scenarios: 9 need fixes, 2 are questions for the stakeholder, and 2 belong to other specialists. None needs new invariant wording.

**What I checked:**

- Every concept in `/home/user/cairn/docs/domain-model/pins-and-context.md`, read against the hub and every model file it points to.
- All files in `docs/srs/`, `invariants.md` included, and all `features/*.feature`.
- Every requirement id my file cites. Each says what my file says.
- The hub relations that name my concepts. They agree.
- I1, I2, I3, I9 and I10 as they bear on my concepts. They read consistently with the model and the requirements.
- My file itself has no internal contradiction and defines each concept once.

Text marked "(open: OQ-39/40/41)" was not treated as a finding.

### Needs fixing (SRS or scenarios)

1. **Qualifying pin – PIN-01's list of sources is too short.**

  - Where: `docs/srs/05-functional-requirements.md:62`.
  - PIN-01 says qualifying pins come "only from a principal (`parinn pin add` at a principal surface, its configuration or its stamp)".
  - The list leaves out the intent and pin-candidate confirmation. The intent is set by `parinn intent set|revise`, and PIN-06's scenario refuses `parinn pin add --type intent`. Confirmation is `parinn pin-candidate confirm`.
  - Read literally, it contradicts PIN-05, PIN-06, LANE-20, LANE-26 and the model's Pin entry.

2. **Pin candidate – PIN-05's "in automation deployment mode" opens a loophole.**

  - Where: `05-functional-requirements.md:66`, quoting "never by configuration in `automation` deployment mode".
  - The qualifier implies configuration could confirm a pin candidate in `interactive` mode. LANE-26 and the model make confirmation only "its own widening principal act".
  - The scenario row at `features/pins.feature:72` tests automation mode only. That mode detects no candidates, so the row proves nothing.

3. **Qualifying pin – PEER-06's access token carries too little.**

  - Where: `docs/srs/05d-peer-requirements.md:22` and `features/peer.feature:68`.
  - The access token carries "qualifying pins … as signed events keeping their provenance (… `assistant` for a run seat's)".
  - A run seat's pin qualifies only through a stamp, and an edited pin restores its newest version. Neither the stamp events nor the edit events are carried, so the ephemeral node cannot work out what qualifies.

4. **Directed post – no interface writes one.**

  - LANE-12 (`05b-lane-requirements.md:24`) needs directed posts.
  - `room_post` (`09-interfaces.md:57`) and `parinn room post` (`09a-command-line-interface.md:56`) have no target-agent parameter.
  - That breaks VIEW-03 (`05b-room-view-requirements.md:40`): everything the room view lets a person do must also exist in the CLI or MCP.

5. **Summary request – the scenario caps something the SRS does not.**

  - Where: `features/lane.feature:515-516`, which caps the summary request itself at 500.
  - LANE-33, §9.2 (`09-interfaces.md:73-74`) and §9.6 (`:175`) cap only `room_summary_get`.

6. **Budget – the step budget is called a "limit".**

  - Where: CMP-05 (`05-functional-requirements.md:124`) says "Exceeding a limit". The scenario (`features/kernel.feature:56-69`) produces "step budget limit".
  - This merges the step budget into the limits, which the model's Budget entry keeps apart.

7. **Budget – §8.4 renames the restore block limit.**

  - Where: `docs/srs/08-data-and-storage.md:57`.
  - It calls INJ-07's restore block limit one of the "model-token caps".

8. **Working view – "the view" is ambiguous.**

  - Where: `docs/srs/03-rationale.md:13`, "Keep the record separate from the view."
  - "The view" is unqualified and can be read as the room view.

9. **Pin – "pins" used in a non-ParleyInn sense.**

  - Where: `features/engineering.feature:11,13`, "the toolchain is pinned …", "pins the Go toolchain".
  - The step is bound at `cmd/parinn/bdd_engineering_test.go:40`, so the binding must change too.
  - The same title's "stamped" belongs to the acts-and-roles specialist.
  - ENG-01 itself avoids the word ("fixed to an exact version").

10. **Model reply and compaction summary – a scenario title coins its own terms.**

  - Where: `features/provenance.feature:85`.
  - "model-reproducible text" and "harness-summarised text" stand in for the model's terms "model reply" and `harness_text`.
  - The examples include system reminders and a restore block written back, which are not summaries.

### Questions for the stakeholder

11. **Working view is never used.** The SRS says "the model's context" or "an agent's context" instead, for example in LANE-24, LANE-30, VIEW-15, VIEW-17 and OWN-25. Should those places say "working view", or should the model note that those phrases stand for the working view?
12. **Qualifying pin under INJ-05.** "Qualifying pin" is defined per run: the rooms the run has had a seat in. INJ-05 (`05-functional-requirements.md:109`) injects "qualifying pins only" when the run is ambiguous, so it is undefined which rooms' pins go in. Over-injecting would break PIN-10's "to runs that have had a seat in its room".

### For other specialists

- **Record (Kernel):** `03-rationale.md:31` reads "Kernel (§5.8)". The kernel is §5.7; §5.8 is Administration.
- **Acts and roles (Stamp):** "stamped" at `engineering.feature:11` (see item 9).

```json
[
{"file":"docs/srs/05-functional-requirements.md","line":62,"quote":"Qualifying pins MUST come only from a principal (`parinn pin add` at a principal surface, its configuration or its stamp), never a harness skill's call.","term":"qualifying pin","breaks":"Pin entry: qualifying pins also come from the owner's intent principal act (LANE-20, `parinn intent set|revise`; PIN-06's scenario refuses `parinn pin add --type intent`) and from a pin candidate's confirmation (PIN-05, LANE-26); read as exhaustive, PIN-01 excludes both","fix":"SRS PIN-01: \"…only from a principal (a pin or intent it sets at a principal surface, a pin candidate it confirms, its configuration or its stamp), never a harness skill's call.\"; optionally add the two rows to features/pins.feature PIN-01 Examples","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/05-functional-requirements.md","line":66,"quote":"never by configuration in `automation` deployment mode","term":"pin candidate","breaks":"Pin candidate entry and LANE-26: only its principal's confirmation, its own widening principal act, makes it a pin; the mode qualifier implies configuration may confirm in interactive mode; scenario row features/pins.feature:72 tests only automation, where no candidate is detected, so it proves nothing","fix":"SRS PIN-05: \"never by configuration\" (drop \"in `automation` deployment mode\"); features/pins.feature PIN-05: add row \"| interactive | a setting that confirms pin candidates automatically | 1 | 1 |\" (count 0 before manual confirmation already asserted)","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/05d-peer-requirements.md","line":22,"quote":"An access token that names a room to continue MUST carry that room's qualifying pins (PIN-10) as signed events keeping their provenance (`operator` for a device seat's pin, `assistant` for a run seat's)","term":"qualifying pin","breaks":"Pin entry / LANE-32: a run seat's pin qualifies only through a stamp, and a pin restores its newest version; carrying only the pins' own events omits the stamps and edits that make them qualify (same in features/peer.feature:68)","fix":"SRS PEER-06: \"…MUST carry that room's qualifying pins (PIN-10) as the signed events they rest on: each pin's creating and edit events, keeping their provenance (`operator` for a device seat's pin, `assistant` for a run seat's), and every stamp that makes a version qualify…\"; mirror in features/peer.feature:68","kind":"needs-fix","invariant":false,"hard_to_revert":true},
{"file":"docs/srs/09-interfaces.md","line":57,"quote":"`text`, `room` (optional target room, LANE-29); writes a `post`, always untrusted (PRV-01)","term":"directed post","breaks":"Directed post entry / LANE-12 need a post directed to one agent, but neither `room_post` nor `parinn room post` (09a-command-line-interface.md:56) can direct one, contrary to VIEW-03's rule that everything the room view does exists in the CLI or MCP","fix":"SRS §9.2 `room_post`: add \"`seat` (optional: directs the post to that run seat's agent, LANE-12)\"; §9.5: \"`parinn room post <room> [--seat <seat>]`\" (stakeholder to confirm the parameter name)","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"features/lane.feature","line":"515-516","quote":"Then the agent's run seat records a summary request to the facilitator, a room act, for 500 model tokens at most","term":"summary request / room summary","breaks":"LANE-33, §9.2 (09-interfaces.md:73-74) and §9.6 cap only `room_summary_get` by `room_summary.max_model_tokens`; the scenario asserts a cap on `room_summary_request` that no requirement states","fix":"SRS §9.2 `room_summary_request`: \"`size` (model tokens wanted, capped by `room_summary.max_model_tokens`)\" and §9.6: \"cap on `room_summary_request` and `room_summary_get`\"; or change the scenario to assert the cap on `room_summary_get`","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/05-functional-requirements.md","line":124,"quote":"Exceeding a limit MUST return an error, restart the worker, and state that the namespace was lost.","term":"step budget","breaks":"Budget entry: the step budget is a named budget and limits keep their own names; CMP-05 folds the step budget into \"a limit\", and features/kernel.feature:56-69 yields \"the step budget limit was exceeded\" / \"kernel step budget limit exceeded\"","fix":"SRS CMP-05: \"Exceeding the step budget or either limit MUST…\"; kernel.feature CMP-05: column \"bound\" with \"step budget\", \"10 s wall-clock limit\", \"512 MiB memory limit\"; steps \"stating the <bound> was exceeded\" and audit \"kernel <bound> exceeded\"","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/08-data-and-storage.md","line":57,"quote":"The pin budget (PIN-08) and the model-token caps of INJ-07, RCL-03 and","term":"restore block limit","breaks":"Budget entry names INJ-07's bound the restore block limit, distinct from the output caps of RCL-03 and CMP-06","fix":"SRS §8.4: \"The pin budget (PIN-08), the restore block limit (INJ-07) and the output caps of RCL-03 and CMP-06 depend on model-token counts.\"","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/03-rationale.md","line":13,"quote":"**Keep the record separate from the view.**","term":"working view","breaks":"Working view entry; unqualified \"the view\" reads as the room view, a different concept","fix":"SRS §3: \"**Keep the record separate from the working view.**\"","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"features/engineering.feature","line":"11, 13","quote":"Scenario: the toolchain is pinned and release builds are static, trimmed and stamped","term":"pin (and stamp)","breaks":"Pin entry and the hub's 'every word one meaning': \"pinned\"/\"pins the Go toolchain\" use the ParleyInn word in another sense, which ENG-01's own text avoids (\"fixed to an exact version\"); \"stamped\" is the acts-and-roles specialist's","fix":"features/engineering.feature: title \"the toolchain is fixed to an exact version and release builds are static, trimmed and carry VCS information\", step \"Then \\\"go.mod\\\" fixes the Go toolchain with a toolchain directive\"; update the binding at cmd/parinn/bdd_engineering_test.go:40 in the same change","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"features/provenance.feature","line":85,"quote":"Scenario Outline: model-reproducible text no principal stamped, and harness-summarised text, is untrusted","term":"model reply / compaction summary","breaks":"Model entry (model reply) and Compaction summary entry (recorded as `harness_text`); the coined phrases replace model terms, and \"harness-summarised\" misdescribes examples that are no summaries (system reminder, restore block written back)","fix":"features/provenance.feature: \"Scenario Outline: model replies and tool calls no principal stamped, and harness_text, are untrusted\"","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/05b-lane-requirements.md","line":36,"quote":"MUST NOT enter any model's context","term":"working view","breaks":"Working view entry ('whatever is currently in the model's context window') is used nowhere in the SRS or scenarios; the same meaning appears as 'the model's context' / 'an agent's context' (also 05b-room-view-requirements.md:52,54; 05b-lane-requirements.md:42; 05c-principal-and-peer-requirements.md:41; 06-security.md:19,100; 09-interfaces.md:87)","fix":"Stakeholder to rule: either the SRS says \"working view\" at these places, or the model's Working view entry states that \"the model's context\" is the harness's phrase for it","kind":"question","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/05-functional-requirements.md","line":109,"quote":"If ParleyInn cannot determine unambiguously which run a hook belongs to, it MUST inject qualifying pins only.","term":"qualifying pin","breaks":"Pin entry / PIN-10 define qualifying pins per run (rooms the run has had a seat in during the run); with the run ambiguous the set is undefined, and pins of a room only another candidate run had a seat in would breach PIN-10 (scenario features/restore.feature:52-60 does not decide it)","fix":"Stakeholder to rule, then SRS INJ-05, e.g. \"…it MUST inject only the qualifying pins of rooms every candidate run has had a seat in, at least its personal room\"","kind":"question","invariant":false,"hard_to_revert":true},
{"file":"docs/srs/03-rationale.md","line":31,"quote":"→ Kernel (§5.8).","term":"kernel","breaks":"Wrong section reference: the kernel is §5.7 (CMP); §5.8 is Administration. Belongs to the record.md specialist (Kernel)","fix":"SRS §3: \"→ Kernel (§5.7).\"","kind":"other-specialist","invariant":false,"hard_to_revert":false}
]
```

### Instruction feedback

- My instructions name the SRS and scenarios but say nothing about step bindings in `cmd/parinn/bdd_*_test.go`. Saying that a scenario wording fix must also cover its binding would help.
- They don't say how to treat the model's own words appearing in plain-English or outside senses ("pinned toolchain", "the model's context") when the hub's exemption for engineering tooling doesn't clearly cover scenario files. A rule would help.
- They don't say how to judge a concept the model defines but the SRS never uses ("working view"). I raised it as a question; a stated policy would help.
