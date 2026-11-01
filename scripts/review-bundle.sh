#!/bin/bash
# Bundles what the change touches: the review class and dispatch, and
# the checked findings, into the step's outputs (ENG-28).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail.
set -e

# review-run writes each specialist's report here, assembled
# from its run log; the generalists read it, and the gate
# checks its form against the agent's answer.
mkdir -p review/reports
# mdsmith's own catalog of the Markdown patterns its rules cannot
# see until a kind, include or catalog declares them, from main's
# pinned mdsmith; the Markdown specialist reviews against it.
mdsmith help patterns -f json > review/mdsmith-patterns.json
cargo run --locked --quiet -p review-bundle -- --base . --head pr-head \
  --diff pr.diff --consent stakeholder-comments.json --out review > class.txt
cat class.txt >> "$GITHUB_OUTPUT"
{
  echo "checked=$(jq -c . review/checked.json)"
  echo "dispatch=$(jq -c . review/dispatch.json)"
} >> "$GITHUB_OUTPUT"
cat class.txt "review/checked.json" >> "$GITHUB_STEP_SUMMARY"
