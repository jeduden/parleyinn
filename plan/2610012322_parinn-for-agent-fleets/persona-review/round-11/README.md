# Persona review, round 11: blind preference against other pitches

Round 11 gave all nine personas four pitches with names replaced:
Buzz's launch post (A), ParleyInn's 300-word v12 (B), OpenAI's dots launch
post (C) and Amp's "What Are Orbs?" page (D). Each ranked them, said
why, judged length and structure, and named one change per pitch. The
sources and word counts are in the
[pitch comparison note](../../../../research/notes/pitch-comparison/pitch-comparison.md).

## Rankings

| Seat                    | Ranking    | Would try | Deciding sentence                                                     |
| ----------------------- | ---------- | --------- | --------------------------------------------------------------------- |
| Fleet developer         | B, D, A, C | B         | "They come back to your agents word for word after every compaction." |
| Returning owner         | B, D, C, A | B         | "Click through to the lines and the runs behind an outcome."          |
| Reviewer                | B, D, A, C | B         | "A third says 'all tests pass' and you dig through logs to check."    |
| Live collaborator       | B, D, A, C | B         | "Agents take instructions only from you and those you trust."         |
| OSS maintainer          | B, A, D, C | B         | "Agents take instructions only from you and those you trust."         |
| Multi-machine developer | A, B, D, C | A         | "Nothing routes through a third party unless you choose it." (A)      |
| Platform operator       | B, A, D, C | B         | "Everything else is untrusted data."                                  |
| Security officer        | B, C, A, D | B         | "Agents take instructions only from you and those you trust."         |
| Agent                   | B, C, D, A | B         | "They come back to your agents word for word after every compaction." |

Mean rank: B 1.1, D 2.7, A 2.8, C 3.4.

## Why

- **B wins** on structure and promises: the pain paragraph first, then
  short bold verb headers that each answer one pain; the words "word for
  word after every compaction"; the plain trust rule. Every seat judged
  its length right.
- **D** is praised for its imperative one-liners and scannable cards,
  and faulted for no trust story on agent-to-agent, Slack and webhooks.
- **A** is praised for identity per participant and self-hosting
  today, and faulted for length, the manifesto and the executive quote.
- **C** is too long for every seat, and seen as a personal assistant
  with no code, diff or merge; its control section and its anecdotes
  are its best parts.

## Changes taken into B

| Note                                                   | Seats                | Change                                                 |
| ------------------------------------------------------ | -------------------- | ------------------------------------------------------ |
| Say it is not a step between me and my agents          | fleet                | "Your harness, terminal and settings stay as they are" |
| "Commit it" has no merge gate                          | reviewer             | "Commit and land it on your forge as always"           |
| How "those you trust" is decided; how recall is marked | security, OSS, agent | "Trust by key"; recall "marked as untrusted data"      |
| No line for coming back                                | owner                | "Back on Monday?" with every gap flagged               |
| Redaction missing                                      | OSS                  | Secrets redacted before anything is written            |
| Peer to peer is one clause                             | multi-machine        | "Offline-first"                                        |

Not taken: a full peer-to-peer section (multi-machine), an operator
line on metrics and fleet config (operator), signed identity and
approval rules (OSS, reviewer), and a recall story (agent); each would
push the pitch back past the length the stakeholder asked for.
