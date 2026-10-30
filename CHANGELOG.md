---
summary: >-
  Release notes and upgrade notes per version, newest first (ENG-23).
---
# Changelog

Every release gets a section here, newest first, with its changes and
its upgrade notes (ENG-23). ParleyInn follows semantic versioning.

## Unreleased

- Every reviewer component of the layered review, and every other agent
  under `.claude/agents`, is held to evaluation cases, run on the model
  its front matter names by the review of a pull request that changes
  it, weekly against the published scorecard, and on demand on a
  candidate model for a comparison (ENG-32). The `review-eval` tooling
  crate runs them.
- The review of a pull request is run by code, not by an orchestrating
  agent (ENG-28): the `review-run` tooling crate runs every specialist
  the review bundle dispatches, then every dispatched generalist, each
  as its own agent, and writes the review outcome from what they
  answered. The review skill and its evaluation cases are retired.
  A change no rule routes gets one generalist, an agent that did not
  answer runs once more, a change to a stakeholder path blocks until a
  stakeholder comment consents (ENG-21), and every review reads the whole
  diff.
- Redaction now also redacts a secret that a neighbouring match hid from
  a rule's word boundary, such as an AWS access key directly followed by
  a principal's pattern (SEC-08). Its work is bounded by the redaction
  scan limit: a text that reaches it is not written, a marker stands for
  it, and it is counted as `redaction_scan_limited`. Upgrade note: a
  transcript ingested before this change that held such a secret
  redacts differently now, so its next ingest may start a new
  transcript generation; the earlier events stay.
- A pull request that adds a dependency with no license text in its
  source now fails CI (ENG-30). The `license-check` command, run by the
  supply-chain job, checks every crate a release links.
- Upgrade note: the product is now ParleyInn and its executable is
  `parinn`, renamed from `cairn`. The executable, its release assets
  (`parinn-*`, `parinn.spdx.json`), its MCP server name and every
  command's name change, and so do the home (`~/.parinn`), `PARINN_HOME`
  and the other `PARINN_` variables, `parinn:` addresses, the
  `ParleyInn-Room:`, `ParleyInn-Link:` and `ParleyInn-Session:`
  trailers, `.parinn.toml` and `logs/parinn.jsonl`. Scripts and hook
  entries that call `cairn` must call `parinn`. No release shipped under
  the old name, so nothing is migrated.
- Repository bootstrap: the SRS, one pending scenario per requirement,
  CI, a nightly fuzz job and the release pipeline. `parinn version` is
  the only command.
- The invariants I1 to I10 are reworded to the domain model; their
  initial wording is approved in ADR-2610091555.
- I2 trusts user events matched to a prompt, and I3 lets a pin be
  omitted for run ambiguity; approved in ADR-2610112209.
- I10 says rebuilding reproduces derived artifacts' content exactly,
  whatever their storage layout; approved in ADR-2610112004.
- A release-only reusable workflow, `release-build.yml`, attests each
  binary's build provenance (SLSA Build Level 3); the build CI runs
  attests nothing. A release runs only from `main`. The release attaches
  its SBOM, and a last job verifies the published release against each
  signer's exact identity at `refs/heads/main` (ENG-20).
- Every command refuses a home that another OS user owns or can reach,
  or that `home.json` binds to another home id than `PARINN_HOME_ID`
  (SEC-02, SEC-03). The home keeps a hash-chained audit log under
  `audit/` that `parinn verify` checks (OPS-02). New commands: `parinn
  hook`, which fails open and audits, `parinn status`, `parinn doctor`
  and `parinn counter ack` (OPS-03, ENG-04).

### Upgrade notes

- An invariant change is a design change (§1.3), so the release that
  carries this rewording is a new major version.
