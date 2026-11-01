#!/bin/bash
# Writes the reason review-run printed into the job summary (ENG-28).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail.
set -e

{
  echo "### The review could not run"
  echo
  if [ -s review/run.txt ]; then
    cat review/run.txt
  else
    echo "review-run printed no reason; the Review step's log holds why"
  fi
} >> "$GITHUB_STEP_SUMMARY"
