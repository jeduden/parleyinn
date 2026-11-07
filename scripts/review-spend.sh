#!/bin/bash
# Fetches what the earlier reviews of the pull request spent, for
# `review-run --spent-before`, which holds them all to one cost cap
# (ENG-28). Each review keeps what it spent alone, so this sums every
# review's. It only fetches: `review-run previous select --of spend`
# decides which artifacts are trusted, by the rule the previous findings
# are picked by, since an artifact of this name may come from any
# workflow a pull request defines, and leaves out this run's own;
# `review-run previous spend` reads each file and sums them. A spend
# that cannot be listed, a run behind one that cannot be looked up, or
# a file that cannot be fetched or read fails the step, naming why: it
# is never taken for nothing spent (I6). How many earlier reviews it
# summed, and what they spent, it says in the job's summary.
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
: > "$dir/runs.jsonl"
for id in $(jq -r '[.artifacts[].workflow_run.id | select(. != null)] | unique | .[]' \
  "$dir/artifacts.json"); do
  gh api "repos/$REPO/actions/runs/$id" --jq '{id, path, event}' >> "$dir/runs.jsonl" \
    || fail "Run $id, behind what an earlier review of this pull request spent, could not be looked up, so no agent runs."
done
jq -s . "$dir/runs.jsonl" > "$dir/runs.json" \
  || fail "The runs behind what earlier reviews of this pull request spent did not read, so no agent runs."
picked=$(cargo run --locked --quiet --offline -p review-run -- previous select --of spend \
  --pull "$PULL" --except-run "$GITHUB_RUN_ID" \
  --artifacts "$dir/artifacts.json" --runs "$dir/runs.json") \
  || fail "What earlier reviews of this pull request spent could not be told apart, so no agent runs."
ids=$(sed -n 's/^spend_artifacts=//p' <<< "$picked")
skipped=$(sed -n 's/^previous_skipped=//p' <<< "$picked")

spent=0
count=0
files=()
for id in $ids; do
  # The cap is one byte past what `previous spend` accepts, so a file
  # past it is refused and not cut short into one that reads.
  gh api "repos/$REPO/actions/artifacts/$id/zip" > "$dir/$id.zip" \
    || fail "What an earlier review of this pull request spent, artifact $id, could not be fetched, so no agent runs."
  unzip -p "$dir/$id.zip" spend.json | head -c 1025 > "$dir/$id.json"
  files+=("$dir/$id.json")
  count=$((count + 1))
done
note="No earlier review of this pull request kept what it spent, so its budget starts at 0 USD."
if [ "$count" -gt 0 ]; then
  read=$(cargo run --locked --quiet --offline -p review-run -- previous spend "${files[@]}") \
    || fail "What earlier reviews of this pull request spent, artifacts $ids, did not read, so no agent runs."
  spent=$(sed -n 's/^spent_before=//p' <<< "$read")
  note="$count earlier review(s) of this pull request spent $spent USD in all, as artifacts $ids keep."
fi
if [ "${skipped:-0}" != 0 ]; then
  note="$note $skipped artifact(s) of this pull request's spend were refused: from no run of main's review workflow."
fi

echo "::notice::$note"
echo "$note" >> "$GITHUB_STEP_SUMMARY"
echo "spent=$spent" >> "$GITHUB_OUTPUT"
