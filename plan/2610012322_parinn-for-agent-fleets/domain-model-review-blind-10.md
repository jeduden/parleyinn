# Domain model: tenth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the ninth-round decisions were applied (commit c998d5a), against
itself, all of `docs/srs`, the invariants and `features/`, reading
every file from disk. This note keeps the needs-fix findings, each
checked against the files. Each question takes its recommended option,
as the stakeholder asked.

## 1. Questions and decisions

| #   | Question                                                                                 | Found | Decision                                                                                                                                                             |
| --- | ---------------------------------------------------------------------------------------- | ----- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Does a pin have one author, or one per version?                                          | 3/3   | One per version; the pin's author is its first version's and alone edits it, except the intent, whose versions the owner's device seat at that time authors.         |
| Q2  | What tells a principal act from a room act taken at a principal surface?                 | 2/3   | The signer and kind: a principal act is a kind OWN-11 classes, signed by a device key once PRV-10 ships; an act a seat key signs is a room act wherever it is taken. |
| Q3  | Which room acts need no role?                                                            | 1/3   | Create room, join, a join request and leave are checked against admission and the add; a run's personal-room seat is a contributor.                                  |
| Q4  | Does a principal's ask for a join also accept the run's join request?                    | 1/3   | Yes: a join request needs its principal's acceptance unless that principal asked for the join; then only the room's admission applies.                               |
| Q5  | Does "signed through a device key" cover token keys and run seats?                       | 1/3   | No: trust follows acts a device key signs and posts and pins written from a device seat such a key certified (I2, I8, PRV-02, ADR-2610062500).                       |
| Q6  | Is what a peer brings in always untrusted, though a principal's own signed acts are not? | 1/3   | Trusted only as I2 allows; being a peer never makes content trusted (I4, SEC-25, ADR-2610062500).                                                                    |
| Q7  | Do restore blocks fall under "built only from trusted structural fields and ids"?        | 1/3   | No: restore blocks carry qualifying pins and trusted structural fields (I2, ADR-2610062500).                                                                         |
| Q8  | Which act appoints the facilitator?                                                      | 1/3   | Appointing a moderator or the facilitator, widening (`parinn appointment make`).                                                                                     |
| Q9  | Does a stage-one phone need I4 wording?                                                  | 1/3   | No: the tunnel is the principal's own, outside ParleyInn; I4 binds ParleyInn's components, and §1.4 says so rather than "fits I4 as written".                        |
| Q10 | Do "run" and "work" bar every other sense?                                               | 2/3   | Only as a noun and as a capability.                                                                                                                                  |

## 2. Fixes that need no decision

- **Model:** token-key-only device-seat pins excluded from the widening
  pin acts; Event's message is the harness's user input; structural
  fields of an act count as structural, the act keeps its class;
  managed policy is settings; store holds the record, derived
  artifacts and payloads (§8.1); derived artifacts include room state
  and trust levels, and the quarantine set is defined; pin budget; the
  bundle's principal key; a pin candidate's principal; endorsement
  sends the text the principal confirmed; at most one facilitator; the
  watchdog observation is never recorded; a no-run event goes to the
  recording device's seat; held requests within a surface's scope;
  the stage-one phone sits under Principal surface; a principal key
  may certify a service account's principal key.
- **SRS and scenarios:** SEC-32 "MUST NOT reach as trusted"; PRV-08
  and appendix A "typed at the harness's input"; LANE-01's branch
  routing names "the run's latest preceding event that names its
  branch"; "allow for session" → `allow-session`, "session title" →
  harness session title; network addresses qualified (SEC-24, SEC-26,
  security.feature); `parinn:` reference; "enrolled seat key" → a seat
  key that chains to a trusted principal key; "record" for one item →
  create room act, transcript line, events; SEC-08 "redacted text";
  ADM-02 and VIEW-18 "harness configuration"; OWN-11, OWN-12, §4.3 and
  §9.5 carry the token-key-only exception; §9.5 gains verbs for the
  principal acts VIEW-03 needs.

## 3. Optional, not chased

"recall address", "subagent identity", "room id"; keymap "dismiss" of
overlaps; "the person" for a principal in scenarios; "link" as a verb;
"clone" in two senses; command-line options such as `--since`; the
agent file's "configuration key"; I4's core list.

## 4. Invariants

I2, I4 and I8 change, recorded in ADR-2610062500.
