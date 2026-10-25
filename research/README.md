# Research

Evidence behind ParleyInn's design decisions, kept verbatim. Each report
answers one question and cites its sources. The notes under
[notes](notes/) are the raw, sourced findings the reports were written
from.

## Reports

- [Agent session storage beyond git](reports/agent-session-storage-beyond-git.md):
  the merged answer. ParleyInn owns an append-only record and lets git
  carry copies. Code stays on git.
- [Git as agent session storage](reports/git-as-agent-session-storage.md):
  20 systems compared on storing sessions in git, with twelve design
  rules.
- [Custom storage, git and chat interfaces](reports/custom-storage-git-and-chat-interfaces.md):
  ParleyInn owns the record and its keys. Git and chat are opt-in views
  outside the shipped binary, and shredding works by destroying keys.
- [Interesting ideas digest](reports/interesting-ideas-digest.md):
  the top ideas from a sweep of about 175 systems.

## Notes

- [Git as agent session storage](notes/git-as-agent-session-storage/):
  Beads and Gas Town, session-in-repo tools, git-native collaboration
  data, git's limits and real time.
- [Git alternatives for agent sessions](notes/git-alternatives-for-agent-sessions/):
  Cursor Origin, agent-native version control, Entire read from
  source, Hacker News, and per-system lessons.
- [Version control beyond git](notes/version-control-beyond-git/):
  non-git systems, organisations that left git at scale, and
  migration paths and the debate.
- [Custom storage, git and chat](notes/custom-storage-git-and-chat/):
  platforms that combine git, chat and encryption, git interfaces
  over custom storage, chat protocols and their deletion semantics,
  and crypto-shredding in law and practice.
- [Agent session storage sweep](notes/agent-session-storage-sweep/):
  the catalog of 309 systems, with storage-pattern counts, cross-cutting
  findings and lessons, plus every fact sheet in `sheets.json`.
- [Amp orbs](notes/amp-orbs/amp-orbs.md): remote agent machines,
  agent-to-agent threads and multiplayer on Amp's service, compared with
  ParleyInn's lane.
- [T3 Code](notes/t3code/t3code.md): a local GUI over coding agents,
  read from its source: event sourced in SQLite with no tamper
  evidence, remote access and telemetry built in.
- [Multica](notes/multica/multica.md): an issue-centered agent workspace with
  run transcripts, squads and review; compared with ParleyInn's record, trust and
  network boundaries.
- [LobeHub](notes/lobehub/lobehub.md): an agent workspace connecting coding
  agents, Topics, Tasks, Groups and memory; compared with ParleyInn's room, record
  and trust contract.
- [Warp](notes/warp/warp.md): shared agent sessions, inline code review and
  Factory work from issues and pull requests; compared with ParleyInn's
  intent-centered room, forge discussion and trust boundaries.
- [Live-PR pitch review](notes/live-pr-pitch-review/): a blind
  adversarial review of the "live pull request" pitch in three lenses:
  missing process, existing products, and inconsistencies with the
  contract and the evidence.
- [OpenAI agent UI](notes/openai-agent-ui/): how OpenAI's Codex
  surfaces show a live harness, its results and several agents, the
  app-server protocol, and what needs OpenAI's central service.
- [Implementation path](notes/implementation-path/options.md): OQ-32
  in two steps, revision 2. Nine options fill one evaluation frame
  ([constraints](notes/implementation-path/constraints.md)) for an app
  on Linux, Windows, macOS, iOS and Android built by a software
  factory, from sourced notes on terminals, protocols, UI and
  packaging, five-platform shells, phone reach, Windows, Zig, smalt's
  tooling, Bun's compile times and 27 agent products. They are then
  [compared](notes/implementation-path/comparison.md) on twenty-one
  criteria and memory safety.
- [OpenAI dots](notes/openai-agent-ui/dots.md): always-on agents with
  their own cloud computer, launched 29 September 2026, and what
  ParleyInn's pitch and design take from them.
- [Domain-model subset search](notes/domain-model-subset-search/README.md):
  a genetic search, with repairs, for the largest consistent, secure and
  useful subset of the domain model. It holds the feature genes, the
  annotated model versions, the assembly and scoring tools, and every
  blind evaluation so far.
- [SRS specialist review, round 33](notes/srs-specialist-review-33/README.md):
  twelve blind reviewers, one per model file plus the domain-model
  agent, read the SRS and scenarios against the model. Their 129
  findings are bundled into 112 clusters, not yet applied.
