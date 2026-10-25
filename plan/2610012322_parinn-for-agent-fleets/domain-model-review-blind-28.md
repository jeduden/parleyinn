# Domain model: twenty-eighth round of blind reviews, merged

Three blind domain-model agents reviewed the model after the
twenty-seventh-round decisions were applied (commit 697d6cf): one across
the whole model, one on principals, harness facts, places, seats, the
record and pins, and one on acts, trust, git and components. This round
ran as a workflow: two skeptics, one on text accuracy and one on whether
the defect is real, checked every needs-fix finding. All 14 survived.
Each question takes its recommended option.

## 1. Questions and decisions

| #   | Question                                                               | Found | Decision                                                                                                                                |
| --- | ---------------------------------------------------------------------- | ----- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | What is the seat `parinn ingest` starts, and who keeps its key?        | 1/3   | A run seat whose key the core keeps like a device seat's, sealed by the core: the one exception to a witnessed run's run-seat key rule. |
| Q2  | How does managed policy mark a listed service account as relaying?     | 1/3   | Each listing carries the same mark as a certificate; a listing without it counts as relaying, so trust grants are refused.              |
| Q3  | Does the trust policy read `bundle` beside `peer`?                     | 1/3   | No: only whether an event was recorded on this node, ingested or received; `bundle` beside `peer` is a display mark (I10).              |
| Q4  | Is a forked harness resume a new run?                                  | 1/3   | Yes; a resume that keeps its harness session continues the run. An existing assumption covers the session id on resume.                 |
| Q5  | May the token key certify the node's personal-room seats?              | 1/3   | Yes: the node's own personal room joins the token key's limit (PEER-06).                                                                |
| Q6  | Do recall tools other than `room_summary_get` return `summary` events? | 1/3   | No: they leave them out and return only the address; the kernel's built-ins too.                                                        |
| Q7  | May titles and labels reach a model?                                   | 1/3   | Only enveloped, through recall; never in trusted text, an opt-in notice or an envelope marked structural.                               |
| Q8  | Which act turns on the room-view component and the launcher?           | 2/3   | Accepting the configuration that turns it on (ADM-04), widening; turning it off is cut. `parinn ui` and `parinn launch` record no act.  |
| Q9  | May managed policy turn a B1 to B3 component on?                       | 1/3   | No, off only, so I4 stands; ADM-04, SEC-22 and the ADM-04 scenario follow.                                                              |
| Q10 | Is recording a `needs changes` verdict cut?                            | 1/3   | No: neutral, as `met` and `not met` are.                                                                                                |
| Q11 | Is the act-classification gate specified?                              | 1/3   | No mechanism is claimed: the model says every principal act a requirement names is classed there.                                       |

## 2. Fixes that need no decision

- **Model:** "hold window" named beside "held request"; a node clone has a
  device key or a token key; who starts an entry point includes an
  ephemeral node's entrypoint; room status reads the branches' heads,
  landings and check states; **review step** defined; seat kind fixed
  when the seat starts; redaction covers what an export, publish or
  invite review sends; Foreign room covers a room every seat of its
  principal left; Open room, Check and Room trailer reworded; the
  room-view component records browser principal acts.
- **SRS, scenarios and proposed ADRs:** "kept" for keys and memory;
  "the room view" for bare "the view"; `--needs-you` and
  `<principal-key>`; NFR-02 for the hook budget; the two proposed ADRs
  drop "trusted boundary", "owner-run", "Results open" and "run
  component".

Applying the round pushed 05-functional-requirements.md past its token
budget with nothing left to cut but normative text. Sections 5.8 to 5.10
(ADM, MEM, OPS) moved, unchanged and keeping their numbers, to
05a-administration-requirements.md, as 05b and 05c split off before.
The adversarial check also restored ADM-05's "under ADM-06" and RCL-04's
"(§9.3)", and kept a dev server's URL out of the proposed app-shell ADR's
webview.

## 3. Optional, not chased

OWN-18's narrowing to terminals only; `parinn notice` beside "opt-in
notice"; the LANE-25 paired-phone "enrollment" wording; personas U1 and
U2.

## 4. Invariants and reviews

No invariant changes. Still waiting for the stakeholder: the I4
commit-trailer wording, the signing of browser principal acts and I10's
"statuses".
