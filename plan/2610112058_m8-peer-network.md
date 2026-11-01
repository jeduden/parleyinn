---
id: 2610112058
title: "M8: peer network"
status: "🔲"
summary: >-
  Peers enrolled by key: machines and other principals sync rooms
  after any partition with no central service, ephemeral nodes hand
  over their record, and invites, directed posts and endorsement.
milestone-order: 7
model: opus
depends-on: [2610112057, 2609292016]
---
# M8: peer network

## Goal

Let a principal's machines, cloud nodes and other people share rooms
peer to peer, with no central service and nothing trusted by default.

## Context

Milestone M8 of [SRS §12.2](../docs/srs/12-delivery-plan.md), after M5.
Its rows are P2, and the ENG-29 review gates it. REC-18 rises to P0.

Scope, by requirement family:

- PRV-09 and PRV-02's rows that need another node's key, which M1 left
- PEER-01 to PEER-07, PEER-09 to PEER-11, PEER-13
- PIN-11, OWN-08, OWN-17, RCL-10, RCL-11
- LANE-09, LANE-10, LANE-12
- SEC-24, SEC-25, SEC-30
- sync for token-key-only nodes

## Tasks

1. Enrollment, sync and convergence after a partition: PEER-01 to
   PEER-04, SEC-24, SEC-25, PRV-09
2. Ephemeral and token-key-only nodes: PEER-05, PEER-06
3. Invites, the contributor role, directed posts and endorsement:
   LANE-10, LANE-12, OWN-08
4. Foreign-room recall and its visibility: RCL-10, RCL-11
5. Erasure requests and refused segments: PEER-11, SEC-30
6. M8's initial targets (§12.3): freeze each with its evidence,
   removing its † and its §12.3 row

## Acceptance Criteria

- [ ] §12.2's M8 exit criteria hold
- [ ] Every scenario in scope is off @pending and passes
