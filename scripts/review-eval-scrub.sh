#!/bin/bash
# Scrubs what the reviewer evaluation wrote of the Claude token once
# more before the summary reads it or the upload keeps it; a form found
# fails the job. Then the credential goes (ENG-32, ENG-28).
#
# Run by .github/workflows/review.yml; it takes the shell's default, no
# pipefail. Writes `scrubbed` to the step's outputs when the files are
# clean of the token, redacted or never holding it.
set +e
cargo run --locked --quiet -p review-eval -- scrub \
  --credential "$RUNNER_TEMP/claude-credential" --path review-eval
code=$?
set -e
rm -rf "$RUNNER_TEMP/claude-credential"
if [ "$code" -eq 0 ] || [ "$code" -eq 3 ]; then
  echo "scrubbed=true" >> "$GITHUB_OUTPUT"
  if [ -f review-eval/report.md ]; then
    cat review-eval/report.md >> "$GITHUB_STEP_SUMMARY"
  fi
fi
exit "$code"
