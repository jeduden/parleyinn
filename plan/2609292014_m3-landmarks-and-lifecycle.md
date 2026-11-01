---
id: 2609292014
title: "M3: landmarks and lifecycle"
status: "🔲"
summary: >-
  Structural landmarks with tiered roll-up, backup and restore,
  purge with tombstones, and a rebuild identical in content.
milestone-order: 3
model: opus
depends-on: [2609292013]
---
# M3: landmarks and lifecycle

## Goal

Give Claude a short, clean index of every span. Give operators backup
and purge. Rebuild must reproduce all derived state exactly.

## Context

Milestone M3. Scope: LMK P0, ADM-06 to ADM-08 and SEC-14, and three
rows M1 left: ADM-09's landmark row, ENG-06's migration kills and
SEC-08's `parinn purge --run` row.

## Tasks

1. Spans and structural landmarks with sanitized fields: LMK-01 to
   LMK-04
2. Tiered roll-up bounded to O(k log n) blocks: LMK-05
3. Backup and restore: ADM-06
4. Purge with tombstones and secure delete: ADM-07, SEC-14; SEC-08's
   `parinn purge --run` row
5. Rebuild: ADM-08; ENG-06's kills of migration; ADM-09's landmark row
   (`parinn verify` checks the landmark index)
6. M3's initial targets (§12.3): freeze LMK-03's field length,
   LMK-05's tier size and REC-25's segments per day, each with its
   evidence, removing its † and its §12.3 row

## Acceptance Criteria

- [ ] Every scenario in scope passes; none of them is still @pending
- [ ] Recall target met at 10 compactions
- [ ] Rebuild is identical in content on every evaluation store
