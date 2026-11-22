---
n: 12
title: "A read the file system resolves to the diff covers it"
status: "✅"
result: false
---
# Phase 12: a read that resolves to the diff covers it

Requirements. None changes, and no scenario comes off `@pending`.
ENG-28 has the review decide from the `Read` calls of a specialist's
run log whether it read every line of the diff it was given. This
phase has it count every read of that very file, by whatever path.

What happened. The local review of plan 2611211544's phase 2, which
writes §2, §5, §5a and §7 as blocks, named a part of one specialist
partial. It had read its whole diff as
`pr-head/../review/diffs/domain-model-hub.3.diff`, climbing back out
of the pull request's tree it had been reading. The review held a path
that climbs to name nothing, so that no link in that tree makes a file
of the same name stand in for the diff. Resumed, the review kept the
parts the first run had found whole and named two of them partial:
they had read their diffs by absolute paths under the first run's work
directory, which the resumed review no longer runs in.

The rule. A read covers the diff when the file system resolves its
path to that very file. A relative path, and one under the directory
the run's log names it started in, is taken under the work directory.
The file system follows a link before the `..` after it, so a climb
through a link the pull request plants names the file it reaches,
never the diff. No instruction changes, so no evaluation run is
needed.

RED. Unit tests of `review-run`: a read of
`pr-head/../review/diffs/<name>.diff`, relative or absolute, covers
the diff, and a part kept from an earlier run that read its diff under
the directory its run started in is whole. Both fail before the
change. A climb through a link to a planted file of the same name
covers nothing, before the change and after it.

GREEN. The reading module of `review-run` resolves each read before
it holds the read to the diff, and is given the run's log, not its
reads alone.

Gate. The tests of `review-run`, clippy denying warnings, rustfmt and
`mdsmith check .` pass. The drift case that has the review no longer
decide from a specialist's reads what it read moves to the changed
call and is caught, with every case that injects a file this phase
touches. `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
