# Round 6: fleet developer

Target: proposal.md at b7ae7fd. No blocker; the default serves this
seat, and one terminal check everywhere stays a preference for OQ-29.

## Round-5 findings

- Resolved: the M7 exit (answering at the harness prompt clears a held
  request); marking ready is neutral and tested; install shows the
  policy diff; batch-clearing. Stale "nothing leaves the machine" text
  remains in plan 2610022338's plan.md and in the onboarding UX draft.
- Partly resolved: session pins work with only the core, but lane and
  project pins cannot be made or ended (A); `parinn needs` names the
  terminal, refusals of `parinn answer` do not.

## Step costs

- Start lanes and catch up: no added steps.
- Answer a permission: free at the harness prompt; in the view, allow
  once costs one tap per `parinn-ui` launch and allow for session one
  tap each; the phone allows once after its own passkey.
- Stop and redirect: stop needs only the terminal; redirecting by
  typing is free; steering from the view costs the session tap.
- Review and land: ready is free; approve and land one tap each.

## New findings

A. Important: with only the core, `/pin` MUST say it lasts only for this
   session, and `parinn install` MUST say lane and project pins need
   `parinn-ui` and an authenticator.
B. Important: for each widening CLI verb, OWN-12 MUST say whether it
   completes from the passkey session or raises a fresh assertion in the
   logged-in tab, and the verb MUST print which and wait, not fail.
C. Important: `parinn lanes` MUST name each running harness's worktree
   path and controlling terminal, for redirecting an agent with no held
   request.
D. Minor: the passkey login MUST last until that `parinn-ui` instance
   stops, or OWN-11 MUST state its expiry as a † value.
E. Minor: `/unpin` naming a lane or project pin MUST say it applies only
   to this session and name the verb that ends the pin lane-wide.

Verdict: serves me as the default stands; A to C before M7.
