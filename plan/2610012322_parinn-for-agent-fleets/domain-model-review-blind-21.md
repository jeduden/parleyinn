# Domain model: twenty-first round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed the domain model after the
twentieth-round decisions and I4's passthrough were applied (commit
620a570), against itself, all of `docs/srs`, the invariants and
`features/`, reading every file from disk. This note keeps the needs-fix
findings, each checked against the files. Each question takes its
recommended option, as the stakeholder asked.

The fixes pushed the one-file model past its budget. On the
stakeholder's choice, the model was split into a hub,
`docs/domain-model/index.md`, and one file per concept group; section 4
records that design step.

## 1. Questions and decisions

| #   | Question                                                                 | Found | Decision                                                                                                                               |
| --- | ------------------------------------------------------------------------ | ----- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | In a foreign room, what of a principal's own content stays trusted?      | 2/3   | Nothing in recall (RCL-10); only its own device-seat pins and the versions it stamped keep restoring, and trust-granted pins stop.     |
| Q2  | Before PRV-10 ships, is changing a restoring device-seat pin a room act? | 3/3   | No: a widening principal act told apart by OWN-02's surface mark; no key certifies a device seat yet, so its pins qualify on its node. |
| Q3  | Which origin do paired-phone events take?                                | 1/3   | `peer`.                                                                                                                                |
| Q4  | May segments be merged and rewritten in storage?                         | 1/3   | Yes, closed segments, without changing any address (REC-25, ADM-07).                                                                   |
| Q5  | May repository configuration shorten retention?                          | 1/3   | No: only the node's principal or managed policy sets it (I1); §9.6's cell says so.                                                     |
| Q6  | Is `TrustedText` a concept?                                              | 1/3   | Yes: **trusted text** (`TrustedText`), made by the core's **restore builder**.                                                         |
| Q7  | May the owner's device seat take principal acts that need a capability?  | 1/3   | Yes: it has every room capability for those too, but a paired phone's.                                                                 |
| Q8  | What are I10's indexes and queues?                                       | 1/3   | The **search index** over event text and each principal's Needs you queue.                                                             |

## 2. Fixes that need no decision

- **Model:** a pin restores to runs that have had a seat in its room
  during the run, the intent by its newest version's author; the
  recorder names the hook handlers, CLI, MCP server or launcher; the
  hook carries its **hook input**, not a payload; landmark blocks;
  the intent cannot be unpinned by its former author; the owner's room
  acts edit pins that do not restore unstamped; device keys receive
  blind-peer segments; Trusted sources follow I2's "once PRV-10 ships";
  the domain-model agent names no settings key.
- **SRS and scenarios:** "`user` turn" → "`user` event" where trust or
  recording is meant (PRV-02, PIN-05, VIEW-07, OWN-02, §9.7.6, appendix
  A and the scenarios); REC-02 "by run"; REC-04 and T9 "hook input";
  LMK-05 and §9.2 "landmark blocks"; LANE-26, OWN-11, OWN-12, §4.3 and
  §9.5 add the pre-PRV-10 device seat; record.feature "provenance
  `unparsed`"; provenance.feature's device-seat post once PRV-10 ships.

## 3. Optional, not chased

The Cut list's "unless it removes a pin" for revocations; layout words
beyond the list; "presence and typing hints"; "merge" beside the room
merge; "controlling terminal", "reason class", "surface mark", "Restore
channel"; the Harness facts and Derived artifact sentences; the stake's
"writes it"; "none is uncertified"; `not-met` beside `not met`;
"certifier" and "Working view" unused; "resume" in two senses; the Names
section's behaviour rules; §6.3's short component names; "config";
"Watch-only"; "cost"; "the run's rooms"; "random per-event secret";
"event log"; "The room records no verdict"; recall of a departed run's
own events; the named limits list.

## 4. The split (design step with the stakeholder)

The stakeholder chose, in this session:

- **Granularity:** one file per concept group, ten files mirroring the
  old headings.
- **Hub:** `docs/domain-model/index.md`, holding the intro, a catalog of
  the concept files, Relations, Names follow the model, Not ParleyInn
  concepts and Changing the model.
- **Lint:** per-file schemas and budgets in `.mdsmith.yml` (the
  stakeholder's consent): the hub's sections are closed, and each
  concept file carries a title, order and summary, with 3000 tokens
  and 250 lines at most. Two drift cases prove the schemas catch a
  dropped hub section and a missing summary (ENG-27).

Links in the SRS, two ADRs, CLAUDE.md, AGENTS.md, the domain-model
agent and the v2 plan now point at the hub or the concept file that
holds the term.
