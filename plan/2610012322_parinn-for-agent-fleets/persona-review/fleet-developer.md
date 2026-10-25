# Persona review: fleet developer

Target: pitch v7, plan 2610012322 (plan, traces, ux/) and plan
2610022338, at 618fafb.

## Blocking

1. A timeout denies my agents, and my own prompt may be unusable while
   the hook waits. attention-steering.md: the default hold is 3 minutes,
   then a deny, under the "Keep going" default. A PermissionRequest hook
   holding for 3 minutes may keep the native prompt from showing. Draft:
   while ParleyInn holds a request, the harness's own prompt MUST stay
   answerable, and ParleyInn MUST NOT deny on a timeout unless the owner
   turned on an away policy.
2. Stopping and redirecting an agent needs a new launch step per agent:
   plain-hook Claude Code has no steer, interrupt or stop; `parinn run`
   is required. Draft: interrupt and stop MUST work from any owner
   surface for a Claude Code session started the normal way, or ParleyInn
   MUST say once, at install, which controls that path lacks.

## Important

3. The UX files contradict each other on what must match everywhere:
   status words, queue order (oldest first versus class then causal),
   key bindings (`x`, `e`, `]` mean different things), and the answer
   command (`parinn answer` versus `parinn approve`). One vocabulary, one
   queue order and one keymap are needed before the SRS change.
4. Two agents editing the same file are flagged only quietly on the
   lane. Draft: when a second open lane edits a file another open lane
   has edited, on any of my nodes, ParleyInn MUST raise an inbox item on
   both lanes at that edit.
5. Answering from a shell needs a code shown only on an owner surface,
   sending me to a second screen.
6. Post notices reach agents by default in one design and not in
   another; recalling a post also raises permission prompts. Draft: a
   lane-post notice MUST NOT reach an agent's context unless the owner
   turned it on for that lane.
7. Nothing says how a lane starts from an issue or a new worktree.
   Draft: a lane MUST come into being from the first hook event in a
   new branch or worktree, with no ParleyInn command.
8. Phase 1 never checks that I keep my speed. The RED test should run
   the NFR-01 hook budgets with the UI open and ten harnesses writing.

## Minor

9. A new token per launch breaks a pinned tab.
10. "Unsandboxed" on every tile; make it one line on Health.
11. Automation mode stays the default on a workstation.

Serves me already: the UI is never required; one answer clears
everywhere; TUI and CLI with --json; install shows the diff first;
imported transcripts marked; badges not writable by agent text.

Verdict: the direction serves me, but the 3-minute auto-deny default,
the per-agent `parinn run` for real steering, and the contradictions
between the UX files would make me give up in the first week.
