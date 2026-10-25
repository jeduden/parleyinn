---
summary: >-
  Release notes and upgrade notes per version, newest first (ENG-23).
---
# Changelog

Every release gets a section here, newest first, with its changes and
its upgrade notes (ENG-23). ParleyInn follows semantic versioning.

## Unreleased

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
