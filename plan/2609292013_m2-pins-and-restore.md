---
id: 2609292013
title: "M2: pins and restore"
status: "🔲"
summary: >-
  Verbatim pins and the deterministic, TrustedText-only restore
  block after compaction, plus plugin install.
milestone-order: 2
model: opus
depends-on: [2609292012, 2609292006]
---
# M2: pins and restore

## Goal

Re-inject pinned constraints verbatim after every compaction through a
byte-deterministic restore block that untrusted content can never reach.

## Context

Milestone M2. Scope: PIN P0, INJ P0, ADM-01 to ADM-03, ADM-10, ADM-17 and
SEC-11, and the last step of REC-02, which M1 leaves `@pending`: a later
run of an agent reaches the agent's rooms through the `rooms` recall scope
and its restore blocks, holding no seat, add or role there until it joins
under LANE-23.

## Tasks

1. Pins as operator events, immutable, budgeted: PIN-01 to PIN-04,
   PIN-06, PIN-08
2. The TrustedText type and the restore builder: INJ-01 to INJ-09,
   SEC-07; REC-02's last step, bound with the own-principal join
3. Project config may only tighten: SEC-11
4. Plugin distribution, install and uninstall, read-only doctor, and
   installed commands free of secrets and home paths: ADM-01 to ADM-03,
   ADM-10, ADM-17
5. M2's initial targets (§12.3): answer OQ-56, then freeze PIN-03's
   pin size, PIN-08's pin budget and INJ-07's restore block limit,
   each with its evidence, removing its † and its §12.3 row

## Acceptance Criteria

- [ ] Every scenario in scope passes; none of them is still @pending
- [ ] The constraint-survival target of §11.1 is met
- [ ] Golden files for the restore block are stable (ENG-10)
- [ ] Install then uninstall round-trips the configuration exactly
