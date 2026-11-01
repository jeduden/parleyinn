#!/bin/bash
# Writes the review's run list on one screen into the job summary; it
# only reports and never fails the review (ENG-28).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail.
set -e

[ -s review/agent-runs.jsonl ] || exit 0
{
  echo "### The review's run list"
  echo
  head -n 1 review/agent-runs.jsonl | jq -r '.review | "verdict \(.verdict): \(.findings_blocking) blocking, \(.findings_nit) nits, \(.gaps) not reviewed, \((((.finished // 0) - (.started // 0)) / 1000) | floor) s"'
  echo
  tail -n +2 review/agent-runs.jsonl | jq -r 'select(.status == "gap") | "- gap: \(.name) part \(.part // "-") attempt \(.attempt)"' | head -n 30
  echo
  echo "The run report, report.html, and the run list, agent-runs.jsonl, are in the review-reports artifact."
} >> "$GITHUB_STEP_SUMMARY" || exit 0
