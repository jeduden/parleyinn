#!/bin/bash
# Writes what the review cost, per agent, into the job summary; it only
# reports and never fails the review (ENG-28).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail.
set -e

[ -s review/usage.json ] || exit 0
cost=$(jq '[.[].cost_usd] | add // 0' review/usage.json || echo unknown)
{
  echo "### What the review cost: \$$cost"
  echo
  echo "| agent | parts | attempts | calls | fresh in | cache write | cache read | out | cost |"
  echo "|---|---:|---:|---:|---:|---:|---:|---:|---:|"
  jq -r '.[] | "| \(.agent) | \(.parts) | \(.attempts) | \(.calls) | \(.fresh) | \(.cache_write) | \(.cache_read) | \(.output) | \(.cost_usd) |"' \
    review/usage.json
} >> "$GITHUB_STEP_SUMMARY" || exit 0
