#!/usr/bin/env bash
# Runs one gate of CI from the one list that defines them,
# scripts/ci-gates.txt (ENG-27): ci.yml calls this for each gate it runs
# and scripts/ci-local.sh runs the same list here, so a command is
# written once. {pkgs} becomes --workspace. The gate's environment, such
# as JOB_INDEX and JOB_TOTAL of a shard, reaches its command.
#
# usage: scripts/ci-gate.sh NAME
set -Eeuo pipefail

gates="$(dirname "$0")/ci-gates.txt"
name="${1:-}"
if [ "$#" -ne 1 ] || [ -z "$name" ]; then
  echo "usage: scripts/ci-gate.sh NAME" >&2
  exit 2
fi
while IFS='|' read -r gate _ _ _ ci _; do
  case "$gate" in "" | "#"*) continue ;; esac
  [ "$gate" = "$name" ] || continue
  case "$ci" in
    @*) echo "ci-gate: $name is defined by its CI job, $ci: nothing to run here" >&2; exit 2 ;;
  esac
  exec bash -c "${ci//\{pkgs\}/--workspace}"
done < "$gates"
echo "ci-gate: no gate $name in $gates" >&2
exit 2
