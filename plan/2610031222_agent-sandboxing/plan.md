---
id: 2610031222
title: "Sandboxed agents: the boundary ParleyInn's owner acts rest on"
status: "🔲"
summary: >-
  Seven residual risks remain while an agent runs unsandboxed as the
  user's own OS user: it can type into the harness, replace ParleyInn,
  delete the record, read the store and pass any software-only check.
  Only a sandbox around the agent removes them. This arm defines what
  ParleyInn needs from a sandbox, proves it against several sandbox
  technologies, and makes ParleyInn detect, record and prefer sandboxed
  agents without depending on any one technology.
model: opus
depends-on: [2610012322]
---
# Sandboxed agents: the boundary ParleyInn's owner acts rest on

## Goal

An agent ParleyInn records cannot reach ParleyInn's store, keys or owner
surfaces. Nor can it reach the owner's terminals and browser, the
harness's configuration or ParleyInn's executables. That holds whichever
sandbox technology the user runs. ParleyInn says on every surface whether
it holds for each session.

## Context

Seven persona-review rounds on plan 2610012322's proposal ended in one
finding. Against an unsandboxed agent of the same OS user, no software
check proves who acted. The proposal now demands a per-act WebAuthn
assertion for every widening owner act. After its threat T21 it lists
seven residual risks that only sandboxing removes (OQ-29):

| #   | Residual risk                                                                 |
| --- | ----------------------------------------------------------------------------- |
| R1  | The agent types into the harness's own prompt to approve itself               |
| R2  | It edits the harness's settings or hooks, or replaces ParleyInn's executables |
| R3  | It deletes or rolls back `PARINN_HOME`                                        |
| R4  | It shows a harmless screen while a widening digest waits for a touch          |
| R5  | It reads every lane straight from the store, unenveloped                      |
| R6  | It uses cut acts (stop, deny, quarantine) against other agents                |
| R7  | It passes the terminal-confirmation opt-in, by design                         |

The stakeholder's decisions (3 October 2026): sandboxing is a separate
arm, and for an unsandboxed session the residual risks are accepted
rather than fought. The fleets proposal carries the second as OWN-22:
each session's sandbox state, and the owner's recorded acceptance of
the risks it leaves open. This arm supplies what OWN-22 rests on.
ParleyInn likely has to support a range of sandbox technologies. The first
two named are NVIDIA OpenShell and Deno's Claw Patrol. The initial
[survey](survey.md) places them and the others.

Two shapes of sandbox exist, and they put the record writer in
different places. A tool sandbox, like Claude Code's and Codex's own,
confines the commands the agent runs. The harness and its hooks stay
outside, so ParleyInn's hooks keep their access. A whole-harness sandbox,
like OpenShell, a container or a microVM, confines the harness and its
hooks too. Then the hook that writes the record runs inside, next to
the agent. Where the writer and its key sit is the arm's central
design question.

Reuse first: ParleyInn builds no sandbox of its own. It integrates with
the sandboxes users already run. It states the properties it needs,
tests them and records the result. The witness runs of OWN-18 need a
sandbox too, and use the same adapters.

Invariants touched:

- I2 and I8 rest on the sandbox for owner acts.
- I4 gains the sandbox's egress policy as the place where the agent's
  own network is limited.
- I9 forbids refusing to record an unsandboxed agent, so ParleyInn records
  and labels it instead.

## Tasks

1. Survey the sandbox technologies and what each blocks: tool
   sandboxes (Claude Code, Codex), whole-harness sandboxes (OpenShell,
   devcontainers, Docker or Podman, gVisor, Firecracker or Kata),
   kernel primitives (Landlock, seccomp, bubblewrap, nsjail, macOS
   sandbox-exec), a separate OS user, egress firewalls (Claw Patrol),
   and cloud sandboxes
2. Write the capability contract: one property per residual risk, a
   way to test each, and where the record writer and its key sit in
   each sandbox shape; in a whole-harness sandbox the agent's tools
   must not reach the writer key, or the writer moves outside
3. Proving slice: a conformance probe that attempts each residual
   attack from inside a sandbox and reports which are blocked, run
   against at least two technologies
4. Detection: ParleyInn records each session's sandbox, its policy digest
   and the contract properties that hold, as `harness_meta`, and shows
   unsandboxed sessions on every surface. Settle who may vouch for a
   sandbox, so an agent cannot claim one it does not run in: the
   launcher, or an adapter outside the sandbox, never the agent
5. Risk acceptance (OWN-22): the install and Setup flow that names
   each open risk, asks again when the set grows, and lets managed
   policy forbid it
6. The SRS change: requirements and `@pending` scenarios for the
   contract, detection and labelling; T21 scoped by sandbox state;
   OWN-22 carried into the SRS
7. Adapters for the supported set: `parinn install` shows, as a diff,
   the sandbox policy that denies ParleyInn's paths, sockets and the
   owner's terminals; witness runs use the same adapters
8. Egress firewalls: read Claw Patrol's or OpenShell's decisions from
   files as untrusted events in the lane, and decide whether their
   human approvals can appear in Needs you

## Execution

| Phase | Model | Gate                                                                                                                                                                          |
| ----- | ----- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1     | opus  | The probe, run inside at least two sandbox technologies with `HOME` and `PARINN_HOME` isolated, reports each residual risk as blocked or open, and the matrix is reproducible |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                    |
| --- | ------ | -------------------------------------------------------- |
| 1   | 🔲     | [A conformance probe for the residual risks](phase-1.md) |
<?/catalog?>

## Acceptance Criteria

- [ ] A capability contract names one testable property per residual
  risk R1–R7, and where the record writer sits in each sandbox shape
- [ ] The conformance probe reports, for each supported technology,
  which residual risks it blocks, reproducibly
- [ ] Every session records its sandbox state, and every surface shows
  an unsandboxed session as such
- [ ] ParleyInn records an unsandboxed agent rather than refusing it (I9)
- [ ] Build-time evidence still shows that no code the core can
  execute opens a socket or starts a program beyond its own kernel
  worker
- [ ] `mdsmith check .` is clean
