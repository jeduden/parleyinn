---
id: ADR-2610112209
title: "I2 and I3 after the rulings on spikes S1 and S3"
status: accepted
summary: >-
  The security reviewer's record, approved on 13 October 2026, of three
  changes to I2 and I3 that the
  stakeholder's rulings of 11 October 2026 on spikes S1 and S3 (OQ-51)
  need: trusted user events matched to a prompt, compaction guidance
  dropped from the closed paths, and pins omitted for run ambiguity.
  Supersedes ADR-2610091555.
---
# ADR-2610112209: I2 and I3 after the rulings on spikes S1 and S3

## Context

Spikes S1 and S3 recorded Claude Code 2.1.296's hook contract and
transcripts. On 11 October 2026 the stakeholder ruled on their findings
(OQ-51):

- ParleyInn installs only the hooks through which it writes to an agent.
  The harness session's MCP server records each run from its transcripts
  (REC-13), so no hook handler records `user` or `harness_meta` events
  any more.
- Compaction guidance (PIN-07) is withdrawn with the `PreCompact` hook.
- After an ambiguous compaction, INJ-05 restores only the pins that
  qualify for every candidate run and names the rest. Spike S13 showed
  that no hook input or transcript names the compacting run while its
  hooks run.

The domain-model and security reviews of writing these in found that I2
and I3, as approved in [ADR-2610091555](ADR-2610091555-initial-invariants.md),
no longer match:

- I2 trusts "the `harness_meta` events its hook handlers recorded, and
  the `user` events they recorded". Read strictly, nothing is trusted
  now; read loosely, trust would rest on a transcript file any process
  of the same OS user can append to.
- I2 lists compaction guidance among the closed paths.
- I3 lets a restore omit a pin only "when the budget omits it".

ENG-29 lets a change to an invariant land only with an accepted record
of a named human security reviewer's approval.

## Decision

[docs/srs/invariants.md](../srs/invariants.md) changes in three places:

1. **I2's trusted sources.** Which `harness_meta` and `user` events are
   trusted changes, as quoted below.
2. **I2's closed paths.** Compaction guidance (PIN-07) is removed. The
   list only narrows.
3. **I3.** A pin may also be named by id and count instead of restored
   when INJ-05 cannot tell which run a restore is for.

I2's sentence on the trusted sources now begins:

```text
The trusted sources are this node's own `operator` and structural events
(hook observations only as PRV-08 allows), the `harness_meta` events its
launcher recorded, and, while the deployment mode is `interactive` and no
run on the node leaves R3 or R8 open, the `user` events matched to a prompt
(PRV-08), and, once PRV-10 ships, …
```

PRV-08 states the rest:

- A `UserPromptSubmit` hook handler keeps a prompt commitment. It is a
  structural hook observation of the hook input's `prompt_id` and a
  commitment to the prompt's redacted text. Its own random key of at least
  256 bits is kept only on this node and erased with the commitment.
- The first transcript line carrying that `prompt_id` uses the commitment
  up. It matches only when it marks the prompt as typed and its redacted
  text matches. The match is recorded, so trust derives from the record
  alone (I10).
- OWN-22 now says a residual risk counts as blocked only by a sandbox state
  the measuring component signed with a key no agent can read. Until OQ-40
  provides that key, every run leaves R3 and R8 open, and so do processes
  of the OS user that no run of this node records. So no `user` event is
  trusted, and hook observations are untrusted.
- The condition is checked in causal order, between a prompt commitment
  and its match, a run with no recorded end counting as active (I10).
- Only the launcher's `harness_meta` is trusted. `harness_meta` a hook
  handler records or a transcript line holds is not.

The text in force is the one invariants.md holds in the commit that adds
this record. Its SHA-256 is:

```text
f23fa79e1b9ff27414ce35d90fae1c33764ca2e1f12540aede2dda129ea6ac5c
```

Until a security reviewer is named, the stakeholder holds that role.
The stakeholder, `@jeduden`, approved this record as security reviewer
on 13 October 2026 by a [comment on jeduden/parleyinn#39][approval] naming
the record's id, first of the three records it approves. The comment
came on a later pull request than jeduden/parleyinn#38, which adds this
record and had merged. It names
neither the hash above nor a commit, so the hash binds the approval to
the text in force when this record was added. That comment is the
approval ENG-29 asks for, and it can be checked apart from this record.

[approval]: https://github.com/jeduden/parleyinn/pull/39#issuecomment-6149850310

## Alternatives

- **Trust what capture recorded live, with no prompt match.** The
  smallest wording change, but trust would rest on a file the agent can
  write to through its tools. The stakeholder chose the match.
- **Keep a hook that records `user` events.** That contradicts the
  ruling that ParleyInn installs only the hooks through which it writes.
- **Leave I3 and treat the omission as a budget omission.** The reviews
  would keep flagging it, and the reason for an omission would not be
  stated.

## Consequences

- **Fail closed until OQ-40.** An agent of the same OS user can append
  transcript lines, run a hook handler with an input it made, and write the
  ParleyInn home, so no transcript line, hook observation or unsigned sandbox
  state is unforgeable for it (§6.1, R3 and R8). Until sandbox states are signed
  out of the agent's reach, which needs the launcher (M7) and OQ-40, no `user`
  event is trusted: PIN-05 finds no pin candidates, and a compaction stays
  ambiguous from a subagent's start to the end of its harness session (INJ-05).
  Pins still restore, since they come from principal acts.
- **Small cost where rooms match.** Where every candidate run qualifies for the
  same pins, an ambiguous compaction loses only the landmark index.
- **Seals.** Before seals ship (REC-18, M7), any record in a home the agent can
  write can be forged; the gate holds once both seals and signed sandbox states
  exist.
- **Fails safe.** A prompt the harness rewrites before writing it to the
  transcript stays an untrusted `user` event.
- **Risk acceptance.** R8 joins the residual risks OWN-22 tracks: while a run
  leaves it open, widening acts need the principal's risk acceptance naming it.
- **Supersession.** This record supersedes ADR-2610091555 (ENG-26), and
  the hash above replaces that record's as the approved text.
