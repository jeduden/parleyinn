#!/usr/bin/env bash
# Names the lines a diff adds that an lcov report shows with no hits, as
# path:line, one a line. scripts/ci-local.sh runs it after the unit
# coverage gate: CI's merge wants every line covered by some layer, so a
# line added here that the unit layer misses may still fail there.
#
# usage: scripts/uncovered-added.sh DIFF LCOV [ROOT]
#   DIFF  a zero-context diff (git diff -U0), LCOV an lcov report whose
#   paths sit under ROOT (default: the current directory).
set -Eeuo pipefail
[ "$#" -ge 2 ] && [ -r "$1" ] && [ -r "$2" ] || { echo "usage: scripts/uncovered-added.sh DIFF LCOV [ROOT]" >&2; exit 2; }
awk -v root="${3:-$PWD}/" '
  FNR == NR {
    if ($0 ~ /^\+\+\+ b\//) file = substr($0, 7)
    else if ($0 ~ /^\+\+\+ /) file = ""
    else if ($0 ~ /^@@/ && file != "") {
      split($3, r, ","); n = (r[2] == "") ? 1 : r[2] + 0
      for (i = 0; i < n; i++) added[file, substr(r[1], 2) + i] = 1
    }
    next
  }
  /^SF:/ { sf = substr($0, 4); if (index(sf, root) == 1) sf = substr(sf, length(root) + 1) }
  /^DA:/ { split(substr($0, 4), d, ","); if (d[2] + 0 == 0 && ((sf, d[1] + 0) in added)) print sf ":" d[1] + 0 }
' "$1" "$2"
