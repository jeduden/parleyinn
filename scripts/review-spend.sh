#!/bin/bash
# Fetches what the earlier reviews of the pull request spent, for
# `review-run --spent-before`, which holds them all to one cost cap
# (ENG-28). It only fetches: `review-run previous select --of spend`
# decides which artifact is trusted, by the rule the previous findings
# are picked by, since an artifact of this name may come from any
# workflow a pull request defines, and `review-run previous spend` reads
# the file. A spend that cannot be listed, told apart or read fails the
# step, naming why: it is never taken for nothing spent (I6). What it
# carried, or that it found none, it says in the job's summary.
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail. Writes
# `spent` to the step's outputs, 0 when no earlier review kept one.
set -eu

dir="$RUNNER_TEMP/spend"
mkdir -p "$dir"

# Says why in the log and the job's summary, and fails the step.
fail() {
  echo "::error::$1"
  echo "$1" >> "$GITHUB_STEP_SUMMARY"
  exit 1
}

gh api --paginate "repos/$REPO/actions/artifacts?name=review-spend-pr-$PULL&per_page=100" \
  --jq '.artifacts[]' > "$dir/pages.json" \
  && jq -s '{artifacts: .}' "$dir/pages.json" > "$dir/artifacts.json" \
  || fail "What earlier reviews of this pull request spent could not be listed, so no agent runs."
for id in $(jq -r '[.artifacts[].workflow_run.id | select(. != null)] | unique | .[]' \
  "$dir/artifacts.json"); do
  gh api "repos/$REPO/actions/runs/$id" --jq '{id, path, event}' \
    || echo "::warning::run $id of an earlier review's spend could not be read; its artifact is refused" >&2
done | jq -s . > "$dir/runs.json"
picked=$(cargo run --locked --quiet --offline -p review-run -- previous select --of spend \
  --pull "$PULL" --artifacts "$dir/artifacts.json" --runs "$dir/runs.json") \
  || fail "What earlier reviews of this pull request spent could not be told apart, so no agent runs."
id=$(sed -n 's/^previous_artifact=//p' <<< "$picked")
skipped=$(sed -n 's/^previous_skipped=//p' <<< "$picked")

spent=0
note="No earlier review of this pull request kept what it spent, so its budget starts at 0 USD."
if [ -n "$id" ]; then
  # The cap is one byte past what `previous spend` accepts, so a file
  # past it is refused and not cut short into one that reads.
  gh api "repos/$REPO/actions/artifacts/$id/zip" > "$dir/spend.zip" \
    || fail "What earlier reviews of this pull request spent, artifact $id, could not be fetched, so no agent runs."
  unzip -p "$dir/spend.zip" spend.json | head -c 1025 > "$dir/spend.json"
  read=$(cargo run --locked --quiet --offline -p review-run -- previous spend "$dir/spend.json") \
    || fail "What earlier reviews of this pull request spent, artifact $id, did not read, so no agent runs."
  spent=$(sed -n 's/^spent_before=//p' <<< "$read")
  note="Earlier reviews of this pull request spent $spent USD, as artifact $id keeps."
fi
if [ "${skipped:-0}" != 0 ]; then
  note="$note $skipped artifact(s) of this pull request's spend were refused: from no run of main's review workflow, or from a run that could not be read."
fi

echo "::notice::$note"
echo "$note" >> "$GITHUB_STEP_SUMMARY"
echo "spent=$spent" >> "$GITHUB_OUTPUT"
