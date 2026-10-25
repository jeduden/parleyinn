---
n: 3
title: "The core prints its target's license file; the release build checks it"
status: "✅"
result: false
---
# Phase 3: the core prints its target's license file

Requirements. Closes ENG-31. Takes the `@ENG-31` scenario off
`@pending`.

BDD coverage: the steps of phase 2. They also run the built core's
`parinn licenses`. They compare its output with the committed file of
the host's release target.

Done, red then green:

- `crates/parinn/src/licenses.rs` embeds the committed file of the
  release target on the build's operating system and architecture with
  `include_str!`. `parinn licenses` prints it.
- A unit test per host compares the embedded text with that target's
  committed file. The scenario checks that the source selects and names
  every target's file.
- The core gains no dependency and no socket.
- The release build (`build.yml`) regenerates the files with
  `license-check --out`. It runs `scripts/check-licenses.sh` on each
  executable on its own platform: the Linux x86_64 and Darwin builds,
  the linux/arm64 job and the Windows job.
- The release build fails when the output differs.
- The only executable the repository ships is the core.

Drift cases (ENG-27):

- The release build no longer regenerates the files.
- The core no longer selects a target's file.
- The core no longer answers `parinn licenses`.
