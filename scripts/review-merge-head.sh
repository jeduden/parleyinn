#!/bin/bash
# Moves pr-head to the pull request's head merged into the commit main's
# checkout, the working directory, holds, and writes pr.diff from main to
# that merge, so the review reads the pull request as landing it would
# leave main (ENG-28). A head that holds main is its own merge. Read
# alone, a head that merged main and another pull request apart has two
# merge bases with main once main merged that one, and a three-dot diff
# reads that one's change as its own; a head behind main differs from
# main's tree in each change main made since. A head that does not merge
# with main is not reviewed: the script fails, naming the files that
# conflict.
#
# Run by the review and evaluate jobs of .github/workflows/review.yml
# and by scripts/review-local.sh, each from the work directory that
# holds main's checkout and pr-head.
set -e

main=$(git rev-parse HEAD)
head=$(git -C pr-head rev-parse HEAD)
merge=$head
if ! git -C pr-head merge-base --is-ancestor "$main" "$head"; then
  set +e
  out=$(git -C pr-head merge-tree --write-tree --name-only "$main" "$head")
  code=$?
  set -e
  if [ "$code" -eq 1 ]; then
    files=$(printf '%s\n' "$out" | sed -n '2,/^$/p' | sed '/^$/d' | paste -sd ' ' -)
    echo "::error::the head does not merge with main, so it is not reviewed (ENG-28): $files conflict; merge main into the branch"
    exit 1
  fi
  if [ "$code" -ne 0 ]; then
    echo "::error::the head could not be merged into main (git merge-tree exit $code)"
    exit "$code"
  fi
  # A fixed author and date make the merge a function of its tree and
  # parents, so the evaluation's job makes the same commit.
  export GIT_AUTHOR_NAME=review GIT_AUTHOR_EMAIL=review@localhost
  export GIT_COMMITTER_NAME=review GIT_COMMITTER_EMAIL=review@localhost
  export GIT_AUTHOR_DATE=2026-01-01T00:00:00Z GIT_COMMITTER_DATE=2026-01-01T00:00:00Z
  merge=$(git -C pr-head commit-tree "$(printf '%s\n' "$out" | head -n 1)" \
    -p "$main" -p "$head" -m "the head merged into main")
  git -C pr-head checkout -q --detach "$merge"
fi
git -C pr-head diff "$main" "$merge" > pr.diff
