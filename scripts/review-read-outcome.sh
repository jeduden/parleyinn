#!/bin/bash
# Reads review/outcome.json and review/consulted.json into the step's
# outputs and the job summary (ENG-28).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail.
set -e

outcome=$(jq -ce . review/outcome.json)
consulted=$(jq -ce . review/consulted.json)
{
  echo "outcome=$outcome"
  echo "consulted=$consulted"
} >> "$GITHUB_OUTPUT"
{
  cat review/run.txt
  echo "consulted: $consulted"
} >> "$GITHUB_STEP_SUMMARY"
