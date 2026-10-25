---
n: 13
title: "Recall and the envelope: the six recall tools, scopes and recall events"
status: "🔳"
result: false
---
# Phase 13: recall and the envelope

Requirements. RCL-01, RCL-03, RCL-04, RCL-05, RCL-07, RCL-08, SEC-06 and
SEC-07, on what phases 8 to 12 record. ENG-10's envelope golden files,
the envelope's round-trip property and its encoder's fuzz target. RCL-03's
8,000-token cap freezes (§12.3), in a docs-only pull request. I2: the
persona-security-officer reviews the envelope and the recall tools.

- The estimator of S8 (§8.4) moves into the `envelope` crate with the
  committed sample under `testdata/`, and a test of its 99% bound.
- `event_expand`, `landmark_list`, `pin_list` and `stat_list` join the
  tools. `event_expand` takes a range, as one string or `from` and
  `to`, in any shown form, and returns whole events in the envelope up
  to 8,000 estimated tokens, then a `next_cursor`; an event past the
  cap is cut at a character boundary and continues from the cursor.
  A cut counts `recall_truncated`. No landmark or pin is recorded before
  M2 and M3, so those two answer an empty envelope. `stat_list`
  answers a structural envelope: counts alone.
- Scopes (RCL-05). `run` is the calling run's writers, found through its
  tool-use id; `room` and `rooms` need the explicit parameter, serve
  only rooms the run has a seat in, and each such call appends an audit
  entry. Without a session binding `run` stays refused.
- Every call that returns an envelope appends a recall event, kind
  `recall`, provenance `assistant`, to the device seat's writer: the
  tool, its redacted arguments, the run and the returned addresses.
  `recall_calls` is read from the record by `stat_list`, as a count, and
  no audit counter, which would fail `parinn doctor` on every recall.
- Writer labels (RCL-08): a label assignment is a structural event in
  the device seat's writer, made when an address is first asked for; a
  label is `A` and its assignment's rank. Short, range, full and ASCII
  forms all resolve. `parinn event search|expand` prints the tool's
  envelope with each control, escape, carriage return and
  direction-override character shown as a placeholder naming its code
  point.
- SEC-07: a static check refuses `TrustedText` built outside the
  `restore_block` crate, with a drift case (ENG-27).

Where scenarios go live: `@RCL-01`, `@RCL-03`, `@RCL-04`, `@RCL-07` and
`@SEC-06` over `parinn mcp`; `@RCL-05` and `@RCL-08` on worlds the steps
build in the store; `@SEC-07` on a fixture workspace. A scenario that
cannot pass as written stays `@pending` and is raised.

Gate: `@RCL-01`, `@RCL-03` to `@RCL-05`, `@RCL-07`, `@RCL-08`, `@SEC-06`
and `@SEC-07` pass or are raised; the envelope's golden files match.
