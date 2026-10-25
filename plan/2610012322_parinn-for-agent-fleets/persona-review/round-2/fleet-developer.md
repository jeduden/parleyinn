# Re-review: fleet developer

Target: proposal.md at ff6fa8e.

## Earlier findings

- Resolved: #1 timeout deny and native prompt (D4, OWN-06, OWN-07,
  NFR-01 hold row; still depends on ASM-18, see N1), #3 UX
  contradictions (§8.1, §8.4, §8.7, §8.8, VIEW-14), #4 overlap
  (LANE-13), #6 notices off (INJ-10), #7 lane from worktree (LANE-01).
- Partly resolved:
  - #2 parinn-run (OWN-15 states missing controls). Drafts: for a control
    shown unavailable, the view MUST show where the harness runs
    (worktree path, tty or pane); parinn-run MUST work as a drop-in
    wrapper set up once, with no per-launch step.
  - #5 shell answers: `parinn answer` still needs parinn-ui open. Draft:
    `parinn answer <id> deny` MUST NOT require the confirmation code.
  - #8 speed gate: the plan-file change is only recommended; the
    Notification hook has no NFR-01 budget row.
- Accepted as not adopted: #9 pinned tab, #11 automation default.

## New findings

N1. Blocking if spike S9 fails: remote answering rests on unverified
    ASM-18, and OQ-17 could remove hook decisions. Draft: if the
    installed path cannot answer held requests from another surface,
    `parinn install` MUST say so, and OWN-15's list MUST include it.
N2. Important: OWN-11 must state whether allow-once and allow-for-session
    count as loosening a rule, or every session-allow needs WebAuthn.
N3. Important: overlap items in Q2 cannot be dismissed (VIEW-05). Draft:
    an overlap item MUST clear on an owner acknowledgement and re-raise
    only for a new file.
N4. Important: agents that run `git checkout -b` stay in the default
    lane. Draft: when a session's branch changes, ParleyInn MUST assign its
    later events to that branch's lane, deterministically from recorded
    harness_meta, with no owner act.
N5. Important: "Ready for review" has no structural source. Draft: it
    MUST be derived only from a recorded owner or agent hand-to-review
    event.
N6. Minor: LANE-13 misses edits made through Bash; use REC-20 checkpoint
    diffs too.
N7. Minor: `parinn open <address>` is a core verb that cannot reach
    parinn-ui; say how it focuses the view.
N8. Minor: `Esc Esc` interrupts while `Esc` closes sheets; move
    interrupt to a key that is not repeated.

Verdict: now serves me; hinges on S9 and on steering not costing a
launch step per agent.
