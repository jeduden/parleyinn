# Round 4: security officer

Target: proposal.md at ca209ea. Most round-3 findings resolved; one new
blocker from the R3-1 change.

## Round-3 findings

- Resolved: revocation reaching issued keys (PRV-10); erasure copies,
  backups, SEC-31 at P0, purges replicated (SEC-31, ADM-06, PEER-11);
  lanes served only to members (PEER-02); export as an owner act
  (SEC-26); recall-only between harnesses (plan 2610022338's phase 1);
  terminal signals; publish bind; rows 24 to 26; token minting under
  OWN-11; document defects.
- Partly resolved: the confirmation code is bound and never stored, but
  R3-1 made the terminal path the default guard (A); the pitch dropped
  "tamper-evident" but §1.2 still says it; receipts are checkable
  elsewhere, but nothing gets them off the machine (C).

## New findings

A. Blocking: R3-1 reopens T21. Any same-user process can open its own
   pty (`script -qc`, Python `pty`, `expect`), become the controlling
   terminal and type a confirmation for a verb it chose; it can also
   kill `parinn-ui` to fall back to typing. A prompt-injected agent can
   then loosen its own rules, pin injected text that restores to other
   agents, release quarantines, purge its trail or export. Narrowest
   safe rule: typing at the terminal suffices only for acts that cut
   what agents can do (deny, interrupt, pause, stop, quarantine,
   revoke); pins in core-only mode go through `/pin` at the harness's
   own prompt; every other act needs an authenticator, or an opt-in in
   root-owned managed policy. Draft: without an enrolled authenticator,
   an OWN-11 act, `export`, `project bind` or `answer allow-session`
   MUST be refused from the CLI unless managed policy explicitly
   permits terminal confirmation, and a stopped `parinn-ui` or
   `parinn-run` MUST NOT lower the confirmation an act needs. T21's
   vector MUST name pty allocation and signalling ParleyInn's processes.
B. Important: plan 2610022338's phase-1 RED test MUST show that a
   same-user process without the launch credential, or driving the CLI
   through a pipe or a pty it controls, cannot complete any act that
   widens what an agent may do.
C. Important: the writer key is in practice a `0600` file and the
   receipt path is writable by the same user, so an agent can rewrite,
   re-seal and rewrite the receipt. Draft: VIEW-10 MUST warn when the
   receipt path is writable by the tenant's user, and MUST show the
   receipt's heads as a short code the owner can carry off the machine.
D. Minor: SEC-10 MUST name, per OS, a key store the core can reach
   without a socket, cgo or a child process, or say the key is a file
   there; `parinn reject` is missing from OWN-12; §1.2 MUST qualify
   "tamper-evident".

Verdict: serves me except A; restrict terminal-only confirmation to
tightening acts and `/pin`.
