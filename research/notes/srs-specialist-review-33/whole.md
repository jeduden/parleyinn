## Domain-model standing review, round 33: SRS (docs/srs/*, invariants.md included) and features/*.feature at HEAD

I read the hub and all ten concept files, then every SRS file and all 18 feature files.

**Excluded terms: clean.** I searched for every excluded term in the model's table. Each hit is in a place the table allows:

- "harness session" and the harness's own names (`SessionStart`, `session_id`, `allow-session`); `session-index` is a third-party tool's name.
- `~/.claude/projects`, `--scope project`, and ParleyInn's own software project (OQ-09, ENG-25).
- `operator` appears only as the provenance class; "structured operators" and "FTS5 operator" are ordinary search words.
- LANE ids and the lane file names.
- "hiding commitment" (REC-17).
- "file-owner" (T6) and "code owner" (ENG-21) are qualified by their own domains.
- "own run" (record.feature:24) is plain English, not an evidence class.

Every use of "token", "request", "link", "receipt", "source", "notice" and "boundary" is qualified or allowed. Every MCP tool, CLI verb, option and settings key follows "Names follow the model". The principal-act classes in OWN-11 and §9.5 match Acts and roles row for row.

### Findings (needs-fix)

1. **LANE-15 treats every foreign room as a bundle room.** (docs/srs/05b-lane-requirements.md:27)

  - The Foreign room entry also covers a room every seat of the principal has left, or lost to a kick or a bar. Such a room has no bundle's principal key and no pull request.
  - LANE-15's first MUSTs therefore cannot be met for those rooms. The scenario (lane.feature:207-220) tests only an imported bundle.

2. **PRV-02's list of trust-level inputs leaves out one the model names.** (docs/srs/05-functional-requirements.md:47)

  - The Trust level entry lists "whether the event's room is foreign to that principal".
  - PRV-02's list omits it, although the same row's own foreign-room rule depends on it.

3. **LANE-01's routing MUST has no ingest exception.** (05b-lane-requirements.md:14)

  - The Relations section says what `parinn ingest` appends for a witnessed run goes *instead* to the seat ingest starts beside the run seat.
  - LANE-01 sends a run's events to "its run seat ... else to its personal-room seat". Read literally, this conflicts with REC-19.
  - The fix is wording only, but routing decides which writer holds an event, and that cannot be undone once recorded.

4. **"unsigned" is used in a second sense.** (features/provenance.feature:37, "this node's unsigned trusted sources")

  - The model uses "unsigned" only for events after the newest seal (Seal entry; the integrity status in §9.7.5).
  - The events meant here are sealed. "Unsigned" here means "signed by no device key".

5. **§9.5 describes publishing as a send to a host.** (09a-command-line-interface.md:37)

  - The Export entry defines publishing as serving a bundle read-only through the publish component. §6.3 row 18 gives that component an inbound, read-only listener.
  - "through the publish component (B3) to a host" reads as an outbound send. Under I4, outbound exchange with hosts belongs to the bridge component.

6. **§9.7.4 speaks of an edited pin version.** (09b-lane-vocabulary.md:82)

  - "a pin version you stamped that was edited, unpinned" contradicts the model: a pin version is immutable, and edits, unpins and list removals act on the pin.

7. **VIEW-21 gives agents a status.** (05b-room-view-requirements.md:58, "agents are all idle"; lane-view.feature:218, "its agents go idle")

  - Idle is a run status from §9.7.1. Agents have no status.

8. **OWN-27 makes the room the actor.** (05c-principal-and-peer-requirements.md:43, "The room records no verdict of its own")

  - A room is a place, not an actor. The model says ParleyInn never derives a verdict, and VIEW-22 says ParleyInn records none.

9. **A crate name leaves out "harness".** (10-engineering-quality.md:17, ENG-02 `adapter-claudecode`)

  - Crate names follow the model, and the model's word is "harness adapter".

10. **ADR-06 says "settings" where it means the harness's settings.** (04-reference-architecture.md:157, "`parinn install` edits settings")

  - In the model, configuration means "the principal's settings". The harness's settings are part of harness configuration.

### Question for the stakeholder (a gap in the model)

11. **"origin" has two meanings, and the model uses both.**

  - In record.md:78, Origin means how an event reached the record (`witnessed`, `ingested`, `bundle`, `peer`).
  - In components-and-surfaces.md:35, "origin" is the browser's sense ("only its own origin, port included").
  - The SRS follows the model in both senses: SEC-20, OWN-11, OWN-16, VIEW-22, T15, OQ-38 and the related scenarios use the browser sense.
  - This breaks the hub's "every word one meaning" rule. The hub's list of outside things that keep their own names (network or email address, flag, code owner, persona name) does not include the browser's origin.

None of these needs new invariant wording. No text marked "(open: OQ-39/40/41)" contradicts its mark.

```json
[
{"file":"docs/srs/05b-lane-requirements.md","line":27,"quote":"A foreign room MUST open in the Room page, marked foreign, which MUST state that every event, evidence class and proof class in it is asserted by the bundle's principal key","term":"foreign room","breaks":"Places → Foreign room: it includes a room every seat of its principal has left or lost to a kick or a bar. Such a room has no bundle's principal key and no pull request, so LANE-15 narrows the concept to imported-bundle rooms and its MUSTs cannot be met for the rest","fix":"SRS LANE-15: \"A foreign room MUST open in the Room page, marked foreign; one imported from a bundle MUST state that every event, evidence class and proof class in it is asserted by the bundle's principal key, and MUST show whether …\"","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/05-functional-requirements.md","line":47,"quote":"Trust levels MUST derive per event, principal and reading node from provenance, origin (reading `bundle` as `peer`), recorder, writer, recorded deployment mode, the node's key set and that principal's stamps and trust grants","term":"trust level","breaks":"Trust and flow → Trust level lists 'whether the event's room is foreign to that principal' among the inputs; PRV-02's list omits it, though PRV-02's own foreign-room rule depends on it","fix":"SRS PRV-02: insert \"whether the event's room is foreign to that principal,\" before \"the node's key set\"","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/05b-lane-requirements.md","line":14,"quote":"Each event MUST be written to exactly one seat's writer: a run's to its run seat in the room it works in, one it joined or created that names the branch of the run's latest preceding event naming its branch, while that seat is a member with work (its role, no mute) in this node's room state on recording it; else to its personal-room seat, never refused.","term":"run seat / the seat ingest starts","breaks":"Relations: 'what `parinn ingest` appends for a witnessed run goes instead to the seat ingest starts beside that run seat (Run seat)'. LANE-01 has no such exception, so its MUST conflicts with REC-19 for ingest appends, including those beside the personal-room seat","fix":"SRS LANE-01: after \"never refused\" add \"; what `parinn ingest` appends for a witnessed run goes instead to the seat ingest starts beside that run seat (REC-19)\"","kind":"needs-fix","invariant":false,"hard_to_revert":true},
{"file":"features/provenance.feature","line":37,"quote":"the default trust policy trusts this node's unsigned trusted sources only on this node","term":"unsigned","breaks":"Record → Seal ('Events after the newest seal are unsigned') and the `unsigned` integrity status in §9.7.5. This node's operator, structural, harness_meta and user events are sealed; here 'unsigned' means 'signed by no device key', a second sense","fix":"Scenario title: \"the default trust policy trusts this node's own trusted sources only on this node, …\"","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/09a-command-line-interface.md","line":37,"quote":"publish a room's bundle through the publish component (B3) to a host the node's principal names","term":"publish","breaks":"Record → Export: 'To publish is to serve a room's bundle read-only through the publish component'; §6.3 row 18 gives it an inbound, read-only listener. 'to a host' reads as an outbound send, which under I4 is the bridge component's outbound exchange with hosts, not publishing","fix":"SRS §9.5 row: \"publish a room's bundle, served read-only by the publish component's listener (B3) at a host the node's principal names, …\"","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/09b-lane-vocabulary.md","line":"82-83","quote":"a pin version you stamped that was edited, unpinned","term":"pin version","breaks":"Pins and context → Pin version: 'One immutable text of a pin; each edit adds one'. Edits, unpins and list removals act on the pin, never on a version (Stamp, LANE-32)","fix":"SRS §9.7.4: \"a pin whose version you stamped was edited, unpinned or taken off the pin list (LANE-32)\"","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/05b-room-view-requirements.md; features/lane-view.feature","line":"58; 218","quote":"A room whose agents are all idle while criteria have no verdict MUST raise a Q3 item (§9.7.4).","term":"Idle (run status)","breaks":"Trust and flow → Run status: a status from §9.7.1, Idle included, belongs to a run; agents carry no status (the scenario repeats it: 'When its agents go idle')","fix":"SRS VIEW-21: \"A room whose runs all show Idle while …\"; scenario: \"When its runs all show Idle and …\"","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/05c-principal-and-peer-requirements.md","line":43,"quote":"The room records no verdict of its own (VIEW-22).","term":"room / verdict","breaks":"Places → Room is a place, not an actor (nodes record); Git and forge → Verdict says 'ParleyInn never derives one', and VIEW-22 says ParleyInn records none","fix":"SRS OWN-27: \"ParleyInn records no verdict of its own (VIEW-22).\"","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/10-engineering-quality.md","line":17,"quote":"`adapter-claudecode`","term":"harness adapter","breaks":"Components and surfaces → harness adapter; hub 'Names follow the model': crate names use the model's words, and 'adapter' alone is not one","fix":"SRS ENG-02: rename the example crate `harness_adapter_claude_code` (matching `restore_block`'s form)","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/04-reference-architecture.md","line":157,"quote":"`parinn install` edits settings only as a fallback.","term":"settings / harness configuration","breaks":"Record → configuration is 'the principal's settings (§9.6)'; the harness's settings are part of harness configuration (Principals and agents → Managed policy). Unqualified 'settings' reads as ParleyInn's configuration","fix":"SRS §4.5 ADR-06: \"`parinn install` edits the harness's settings only as a fallback.\"","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/domain-model/record.md; docs/domain-model/components-and-surfaces.md; docs/srs/06-security.md; docs/srs/05c-principal-and-peer-requirements.md; docs/srs/05b-room-view-requirements.md; docs/srs/13-open-questions-and-risks.md","line":"record.md:78; components-and-surfaces.md:35; 06-security.md:56,110,111; 05c:27,32; 05b-room-view:59; 13:45","quote":"exchanged once for an **origin secret** only its own origin, port included, can read or send","term":"origin","breaks":"Hub rule 'every word one meaning': the model defines Origin as how an event reached the record, yet uses 'origin' unqualified in the browser's sense, and so does the SRS (SEC-20, SEC-21, OWN-11, OWN-16, VIEW-22, T15, OQ-38, and the related scenarios). The hub's list of outside things that keep their own names does not include it","fix":"Model (stakeholder ruling): add 'a web origin' to the hub's list of outside things qualified by their own domain and say 'web origin' in components-and-surfaces.md and the SRS rows listed, or state in the Origin entry that the browser's origin keeps its own sense","kind":"question","invariant":false,"hard_to_revert":false}
]
```

### Instruction feedback

- The instructions give no rule for plain-English words that match a concept's name ("fleet load", "idle", "asserted", "scope", "account"). I had to judge each case alone, so the line between a style choice and a departure may differ from one review to the next. One sentence saying when a coinciding word counts would help.
- The steps for checking the model against itself (one definition per concept, no overlapping meanings) are only under "When the model changes". It is not explicit that a standing review runs them too, which is how I found item 11. I reported it as a question, as the reporting section asks.
- The SRS change log and Status row in docs/srs/index.md are historical records under the model's own rule. The instructions do not mention them; one sentence saying to skip them would save searching that file.
