# Re-review: returning owner

Target: proposal.md at ff6fa8e.

## Earlier findings

- Resolved: #1 ids (VIEW-08..11), #3 search (VIEW-09, if D6 holds), #5
  unsigned seal (VIEW-10; but §10.11's pitch line says "signed"), #7
  "tells you when anything is missing" restored, #8 boundary source,
  #9 frontier per writer, #10 thinking blocks searchable, #11 stuck
  computed in the viewer.
- Partly resolved:
  - #2 weekend fixture: in M7's exit, but §11.3 only recommends it and
    never says what the fixture contains (failed capture, gap, late
    ingest, quarantine, purge).
  - #4 failed capture: "capture failed" is undefined, and some failures
    cannot appear (A).
  - #6 one catch-up: VIEW-04/05 cite proposal §8, which nothing makes SRS
    text; Catch up's own line order is unset.

## New findings

A. Blocking: a lane that starts while capture is broken never exists,
   so catch-up cannot show it, and search footers omit capture gaps.
   Draft: Catch up MUST list every harness transcript on the node that
   is not ingested up to its end, and every failure counter that rose
   since the boundary, including sessions that belong to no lane; every
   search result footer MUST name the capture gaps inside the range.
B. Important: nothing proves the record was not altered while away.
   Draft: Catch up MUST check that every writer's chain head recorded at
   the boundary is still a prefix of its current chain, show the result
   per writer, and `parinn receipt make` MUST write outside PARINN_HOME.
C. Important: the board can say Quiet over a dead lane. Draft: a lane
   MUST carry the worst freshness mark of its harnesses on every
   surface, and MUST NOT show Quiet while any harness is unrecorded or
   behind.
D. Important: a D7 veto would take catch-up, search and verify out of
   v1. Draft: the CLI forms of VIEW-08..11 MUST keep their priority
   whatever D7 decides for the browser view.
E. Minor: say "unchanged since its newest seal or receipt" for unsigned
   tails.
F. Minor: cap the first catch-up screen with collapsed per-lane counts.

Verdict: now serves me, but a lane started with capture broken is still
invisible; needs A and B before I trust Monday's catch-up.
