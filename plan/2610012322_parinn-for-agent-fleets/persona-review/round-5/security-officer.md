# Round 5: security officer

Target: proposal.md at a3fa2bc. Two round-4 findings resolved, two
partly; one new blocker.

## Round-4 findings

- Partly resolved: T21. Terminal alone now suffices only for acts that
  cut, the harness prompt serves the core-only case, a stopped surface
  cannot lower the check, and T21 names ptys and signals. But the code
  guarding widening acts can be obtained by the agent (N1).
- Resolved: the phase test covers a pipe, a pty and a missing launch
  credential (not an agent starting its own surface, N1); `parinn
  reject` in OWN-12; SEC-10's key store.
- Partly resolved: VIEW-10 warns and gives a short code, but §1 still
  says "tamper-evident record" with no qualifier, and §10.14 cites a
  "§1.2" that does not exist.

## New findings

N1. Blocking: an agent can start the code's surfaces itself.
    `parinn-ui --print` puts the launch credential on its stdout, and
    `parinn-run -- sh` in a pty it owns shows it the banner code;
    `parinn-ui --device phone` mints a credential "shown only in the
    owner's terminal", which can be a pty the agent controls. Self-
    approval and owner-voice steering (T20, T21) return. Draft: an owner
    act that widens what an agent may do MUST NOT be accepted on a
    credential, code or phone credential from a `parinn-ui` or
    `parinn-run` instance a same-user process could have started, unless
    a WebAuthn assertion backs it or root-owned managed policy permits
    it; T21 and the phase-1 RED test MUST name an agent starting its own
    `parinn-ui --print`, `parinn-ui --device phone` or `parinn-run`.
N2. Important: screen reads (`tmux capture-pane`, `screen` hardcopy,
    `kitty @ get-text`) and keystroke injection (`tmux send-keys`) are
    not in the threat model; an injected `/pin` counts as present and
    restores lane-wide. Draft: T21 MUST name them, and a `/pin` typed at
    the harness prompt MUST stay scoped to its own session unless
    confirmed under OWN-11.
N3. Minor: the glossary, D5, OWN-11 and OWN-12 list different owner
    acts. Draft: they MUST list the same acts, and a gate MUST fail when
    they drift.
N4. Minor: the receipt code MUST carry at least 80 bits of the heads'
    digest.

On OQ-29: its alternative reopens T21; a trail only detects after the
fact.

Verdict: closer, but widening acts rest on a code from surfaces the
agent can start itself.
