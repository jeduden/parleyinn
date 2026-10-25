# Version 6: core repairs from the v5-core review

Folder GA = research/notes/domain-model-subset-search (the caller gives its
absolute path). GA/annotated/ is version 5 (snapshot GA/annotated-v5/); its
files end in `.md.ann`. Follow GA/repair-brief.md and GA/annotate-brief.md.
Findings: GA/evals/v5-core.findings.json (6 needs-fix, 8 minor). Settled by
the stakeholder, keep as is: the principal's own tunnel and `git push` are
the principal's tools outside I4; room trailers are always on.

The v5 repairs at the principal surface and install spawned most of these
findings: added text drew new findings. Prefer one boundary sentence that
settles a family of findings over a clause per finding. Add no concept.

## Needs-fix

T1 Same-OS-user boundary (finding 6, and minor 10): the SRS (§6.1) accepts
   that a run's tool process running unsandboxed as its principal's own OS
   user can do what that user can do, under residual risks R1–R7; only a
   sandbox removes them. Say this once, in Principal surface: ancestry is
   the test ParleyInn applies; a run's tool process that drives a CLI or TUI
   through a process it did not start (a terminal multiplexer, a scheduler,
   keys sent to an open TUI) or writes the store outside ParleyInn's code is
   §6.1's residual risk (R1, R3, R7), which only a sandbox removes. Extend
   the R3 sentence in Principal act to "writes to the store, or signs with
   this node's device key or a seat key, outside ParleyInn's code". Do not
   invent authentication.
T2 Harness-started roles (findings 1, 7, and minor 9): only a process the
   harness itself starts from its configuration runs as a hook handler,
   the MCP server or the harness strip. The same command run from a run's
   tool process or any process descending from it is not one: a hook call
   is refused and counted, and an MCP-server or harness-strip call is
   recall, enveloped. A harness a run's tool process started is such a
   descendant: its hook calls are refused and counted, and its runs show
   as unrecorded.
T3 Uninstall (finding 8, and minor 3): install runs at a principal surface,
   through the harness's own plugin install or through the managed install
   path; uninstall only at a principal surface. Hook registrations found
   removed without that act are counted and shown at the next start of any
   core component as a node-wide capture gap on Runs and Health (I6).
T4 The personal room's creation (finding 0): the first core component to
   run on a node records the create room act of its personal room, from
   the node's device seat, whatever process started it; this is the one
   room act exempt from the principal-surface refusal.
T5 Create room in core (finding 12): create room is taken only once per
   node, creating its personal room; any other create room act is refused
   and counted (I6). Inside the work-rooms span, keep the general create
   room act (nest so that work-rooms candidates read as before).

## Minor (smallest wording change; skip if it needs new text beyond a clause)

- #2: a value in managed policy or repository configuration that is not
  cut-only is not applied; it is counted and shown on Setup (I6), and
  ParleyInn never rewrites the policy itself.
- #4: while capture is off, `parinn ingest` appends nothing for this node's
  witnessed runs.
- #5: settled by T5 (every pin belongs to the personal room in core).
- #11: "kinds" on closed paths means event kinds, seat kinds, pin types and
  provenance classes without their `<tool>` or `<server>` names.
- #13: in Quarantine, say a derived artifact is quarantined by
  quarantining what it derives from or by taking it out of recall
  directly; do not change I5.

Every invariant text change goes in GA/invariant-edits.md (none expected).

## How to work this time

Work on a copy, so the repository never holds a half-edited version: copy
the whole GA folder to WORK = <scratchpad>/ga-v6 (the caller names it),
edit and check there (the scripts use their own folder), and when done,
copy back only WORK/annotated/*.md.ann, WORK/features.json and any new
WORK/genomes/*.json into GA. Then in GA: `python3 check_annotation.py
--no-orig` prints OK; write GA/genomes/v6-core.json and v6-all.json (same
features as v5-core and v5-all) and assemble them into GA/out/. Do not run
git or mdsmith. Report per ruling what you changed (before and after) and
what you skipped and why.
