#!/bin/bash
# Writes the empty finding list and report of a reviewer evaluation with
# nothing to evaluate (ENG-32).
#
# Run by .github/workflows/review.yml.
set -e

mkdir -p review-eval
echo '[]' > review-eval/findings.json
echo "No reviewer component changed; nothing was evaluated." > review-eval/report.md
