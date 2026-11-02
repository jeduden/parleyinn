#!/bin/bash
# Fetches the findings the previous review of the pull request kept, for
# `review-run --previous` (ENG-28). It only fetches: `review-run previous
# select` decides which artifact is trusted, since an artifact of this
# name may come from any workflow a pull request defines, and `review-run
# previous check` refuses a file that does not read. Whatever it finds or
# lacks, it says in the job's summary and goes on: a review with no
# history counts every finding new, and says so (I6).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail. Writes
# `file` to the step's outputs, empty when no earlier findings qualify.
set -e

dir="$RUNNER_TEMP/previous"
mkdir -p "$dir"
file=""

gh api "repos/$REPO/actions/artifacts?name=review-findings-pr-$PULL&per_page=100" \
  > "$dir/artifacts.json"
for id in $(jq -r '[.artifacts[].workflow_run.id | select(. != null)] | unique | .[]' \
  "$dir/artifacts.json"); do
  gh api "repos/$REPO/actions/runs/$id" --jq '{id, path, event}' \
    || echo "::warning::run $id of an earlier review's findings could not be read; its artifact is refused"
done | jq -s . > "$dir/runs.json"

picked=$(cargo run --locked --quiet --offline -p review-run -- previous select \
  --pull "$PULL" --artifacts "$dir/artifacts.json" --runs "$dir/runs.json")
id=$(sed -n 's/^previous_artifact=//p' <<< "$picked")
skipped=$(sed -n 's/^previous_skipped=//p' <<< "$picked")

note="No earlier review of this pull request was found, so every finding is new."
if [ -n "$id" ]; then
  if gh api "repos/$REPO/actions/artifacts/$id/zip" > "$dir/findings.zip" \
    && unzip -p "$dir/findings.zip" findings.json > "$dir/findings.json" \
    && cargo run --locked --quiet --offline -p review-run -- previous check "$dir/findings.json"; then
    file="$dir/findings.json"
    note="Earlier review: the findings of artifact $id."
  else
    note="The findings of artifact $id did not read, so every finding is new."
  fi
fi
if [ "$skipped" != 0 ]; then
  note="$note $skipped artifact(s) of this pull request's name came from no run of main's review workflow and were refused."
fi

echo "::notice::$note"
echo "$note" >> "$GITHUB_STEP_SUMMARY"
echo "file=$file" >> "$GITHUB_OUTPUT"
