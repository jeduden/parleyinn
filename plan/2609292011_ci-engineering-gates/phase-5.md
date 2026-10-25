---
n: 5
title: "SLSA Build Level 3 provenance (ENG-20)"
status: "✅"
result: false
---
# Phase 5: sLSA Build Level 3 provenance (ENG-20)

Requirement. ENG-20 (P0, Ver T): release artifacts are signed with
Sigstore (keyless), carry SLSA Build Level 3 provenance and an SPDX
SBOM, and ship with a signed checksum file.

What exists: `release.yml` builds through the reusable `build.yml`.
It attests provenance in its own publish job, though. So the
provenance's signer is the caller, not the isolated build.

BDD coverage: `@ENG-20` comes off `@pending`. Its steps read the
release workflows. Provenance is attested inside the reusable build
workflow. The SBOM is attested and the checksums signed keyless. A
verification job checks the published release, with the build
workflow as its signer.

RED: the scenario fails on provenance attested outside `build.yml`.
GREEN: the attestation moved, the verification job, a drift case.

Gate: `@ENG-20` passes.

Status: done. The stakeholder ruled on 2026-10-13
([ruling]) to reword the
scenario to what it checks: the release workflows sign, attest and
verify every release they publish. `@ENG-20` is off `@pending` with
that wording.

[ruling]: https://github.com/jeduden/parleyinn/pull/44#issuecomment-6157262395
