# Round 5: fleet developer

Target: proposal.md at a3fa2bc. Three round-4 findings resolved, two
partly, one settled against this seat; no blocker.

## Round-4 findings

- Settled against me, now a preference for OQ-29: N1. Journeys still
  work: answers at the harness's own prompt or in the view, steering by
  typing to the harness.
- Resolved: the Needs you trail for policy-confirmed acts (OWN-12); the
  `parinn-run` totals and keystroke limit (NFR-09); the launch path and
  install opt-in (OWN-15).
- Partly resolved: M7's exit covers only core-only, not answering at
  the harness prompt while `parinn-ui` holds the request; the pitch's
  decision list still says "Standalone means nothing leaves the
  machine".

## New findings

F1. Important: with only the core, a pin can be added with `/pin` but
    never ended or changed. Draft: with only the core installed, ending
    or changing a pin MUST be possible at the harness's own prompt in
    interactive mode, under the rule that makes `/pin` count as present.
F2. Important: with only the core, a lane can never be marked ready.
    Draft: every owner act M7's journeys need (answer, pin, end a pin,
    mark ready) MUST have a path at the harness's own prompt with only
    the core installed, or `parinn install` MUST list it as unavailable
    and name the component that offers it.
F3. Important: for each held request, `parinn needs` and every refusal of
    `parinn answer` MUST name the lane, the worktree path and the
    harness process's controlling terminal.
F4. Minor: `parinn install` MUST show, as a diff, the managed-policy file
    that permits terminal confirmation and the elevated command that
    installs it, and MUST NOT write it itself.
F5. Minor: a Needs you item for a terminal-confirmed act MUST be
    clearable only by the owner from an authenticated surface, in a
    batch, while its Catch up line stays.

Verdict: serves me if OQ-29 keeps the default; F1 to F3 before M7.
