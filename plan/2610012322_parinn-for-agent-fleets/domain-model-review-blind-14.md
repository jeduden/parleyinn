# Domain model: fourteenth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the thirteenth-round decisions were applied (commit 2c57301),
against itself, all of `docs/srs`, the invariants and `features/`,
reading every file from disk. This note keeps the needs-fix findings,
each checked against the files; one claim (the domain-model agent's
"configuration key" naming a concept) was dropped, since the model's own
Names section uses those words. Each question takes its recommended
option, as the stakeholder asked.

## 1. Questions and decisions

| #   | Question                                                               | Found | Decision                                                                                              |
| --- | ---------------------------------------------------------------------- | ----- | ----------------------------------------------------------------------------------------------------- |
| Q1  | Does a delegated task reach another principal's agent under I2?        | 2/3   | Yes, under an acceptance grant that agent's principal recorded (I2, ADR-2610062900).                  |
| Q2  | Which "witnessed" do the trusted sources mean?                         | 1/3   | Witnessed by the hook handlers, as I2 says; origin `witnessed` stays wider (PRV-02, VIEW-07, §9.7.6). |
| Q3  | Does a trust grant reach a foreign room?                               | 1/3   | No, stated in the trust grant itself; PRV-02 and VIEW-07 need no change.                              |
| Q4  | Is dismissing a directed post cut or neutral?                          | 1/3   | Cut; dismissing any other Q3 or Q4 item is neutral.                                                   |
| Q5  | What role does a former owner hold after a handover?                   | 1/3   | Moderator, by a role assignment the handover records (LANE-11).                                       |
| Q6  | Does a service account whose certificate is revoked count as a person? | 1/3   | No: only a principal key never certified counts as a person's.                                        |
| Q7  | Is managed policy the only non-principal source of settings?           | 2/3   | It is the only source that overrides the principal's; repository configuration only tightens.         |
| Q8  | Does joining a room trigger a restore block?                           | 1/3   | No: LANE-30 names rooms in the next restore block, at the points INJ-02 sets.                         |

## 2. Fixes that need no decision

- **Model:** a node, not a device, joins before recording a principal
  act; Join covers a device seat's join without admission; structural
  fields include key fingerprints, addresses and version numbers;
  "reply" and room id defined; a result's text claim is "a claim stated
  in text", and its other marks are §9.7's; the owner's device seat makes
  a list removal of any pin but the intent; Kernel says an agent.
- **SRS and scenarios:** "outcome" outside the Outcome concept →
  "results", "a `harness_meta` event", "a claim stated in text" (VIEW-06,
  VIEW-13, OWN-02, REC-23, lane.feature); LANE-14 "the harness's user
  input"; LANE-22 "role, or owner"; §9.4 and INJ-01 list the pins that do
  not qualify and the room summary pointer; NG9 and §6.3 row 23 "several
  seats editing one worktree"; ADM-15 "free space"; `--principal` for a
  principal (ADM-14, administration.feature, security.feature);
  recall.feature "has recorded a trust grant"; OWN-07 "answer"; OWN-11,
  09b and §9.5 class dismissing a directed post as cut; node joins in
  LANE-01, §4.3, OWN-02 and §9.5.

## 3. Optional, not chased

I4's core list; the CLI's request verbs (`purge-request make` beside
`role request`); `parinn check witness`; "the view"; "untrusted
envelope"; I9's "defined budgets"; "result" in recall and error senses;
"Boundary to durable memory"; `parinn notice`; `--seat`, `--writer` and
`--author` overlapping.

## 4. Invariants

I2 changes, recorded in ADR-2610062900.
