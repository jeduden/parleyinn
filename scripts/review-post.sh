#!/bin/bash
# Posts the gate's review; a post that fails posts the review-gate check
# failed (ENG-21).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail.
set -e

if [ -s review.json ]; then
  jq --arg url "$REPORTS" '.body += "\n\nThe specialist reports: \($url) (artifact review-reports)."' \
    review.json > posted.json
  if ! gh api --method POST "repos/$REPO/pulls/$PULL/reviews" --input posted.json; then
    gh api --method POST "repos/$REPO/check-runs" -f name=review-gate \
      -f head_sha="$HEAD_SHA" -f status=completed -f conclusion=failure \
      -f "output[title]=The review gate could not post its review" \
      -f "output[summary]=Posting the review failed; the log of $REPORTS says why."
    exit 1
  fi
fi
