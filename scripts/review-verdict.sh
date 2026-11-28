#!/bin/bash
# After a stakeholder's comment, looks for the verdict an earlier review
# of the same head kept, so the comment's review runs no agent (ENG-37):
# consent is a comment, and a comment changes only what the review
# bundle's deterministic findings say, which this run states again. It
# only fetches: the earlier reviews are the runs behind the spend the
# step before trusted (scripts/review-spend.sh), runs of main's review
# workflow alone, newest first, and `review-run previous verdict` decides whether
# what the newest one that kept a run id left stands for this head and
# dispatch. A verdict that is reused is copied into review/, with a line
# for the job's summary; anything else, a listing or a file that fails
# included, is said in the job's summary and the agents run (I6).
#
# Run by .github/workflows/review.yml after the earlier spend is fetched,
# which hands it its values in the environment; it takes the shell's
# default, no pipefail. Writes `reused`, and `run`, the earlier review's
# run when it is reused, to the step's outputs.
set -eu

dir="$RUNNER_TEMP/verdict"
mkdir -p "$dir"

# Says what was decided in the log and the job's summary, writes
# `reused` and ends the step.
decided() {
  echo "::notice::$2"
  echo "$2" >> "$GITHUB_STEP_SUMMARY"
  echo "reused=$1" >> "$GITHUB_OUTPUT"
  exit 0
}

[ "$EVENT" = issue_comment ] || decided false "A CI run brought this review, so its agents run."

trusted=$(cat "$RUNNER_TEMP/spend/trusted-runs.txt") \
  || decided false "The runs of this pull request's earlier reviews did not read, so the agents run."

for run in $trusted; do
  kept="$dir/$run"
  artifact=$(gh api "repos/$REPO/actions/runs/$run/artifacts?name=review-reports" \
    --jq '.artifacts[0].id // empty') \
    || decided false "The reports of run $run could not be listed, so the agents run."
  [ -n "$artifact" ] || continue
  gh api "repos/$REPO/actions/artifacts/$artifact/zip" > "$kept.zip" \
    && mkdir -p "$kept" && unzip -q -o "$kept.zip" -d "$kept" \
    || decided false "The reports of run $run could not be fetched, so the agents run."
  # A review that could not run kept no run id: an older one may have.
  [ -f "$kept/run-id.json" ] || continue
  weighed=$(cargo run --locked --quiet --offline -p review-run -- previous verdict \
    --dir "$kept" --head "$HEAD_SHA" --dispatch review/dispatch.json) \
    || decided false "The verdict run $run kept could not be weighed, so the agents run."
  if [ "$(sed -n 's/^reused=//p' <<< "$weighed")" != true ]; then
    decided false "Run $run's verdict is not reused: $(sed -n 's/^why=//p' <<< "$weighed"). The agents run."
  fi
  cp "$kept/outcome.json" "$kept/consulted.json" "$kept/answers.json" \
    "$kept/run-id.json" "$kept/gaps.json" review/ \
    && cp -r "$kept/reports/." review/reports/ \
    || decided false "What run $run kept could not be copied, so the agents run."
  if [ -f "$kept/ignored.json" ]; then cp "$kept/ignored.json" review/; fi
  echo "reused the agents' verdict of run $run on $HEAD_SHA; no agent ran" > review/run.txt
  echo "run=$run" >> "$GITHUB_OUTPUT"
  decided true "The agents' verdict of run $run on this head is reused: only the deterministic findings are stated again, and no agent runs."
done
decided false "No earlier review of this pull request kept a verdict, so the agents run."
