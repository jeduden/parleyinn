# Persona review: open-source maintainer

Target: pitch v7, plan 2610012322 (plan, traces, ux/) and plan
2610022338, at 618fafb.

## Blocking

1. "Re-run here" feeds the contributor's recorded command to my agent,
   contradicting UI-04. Draft: a command taken from an untrusted event
   MUST NOT reach any agent or terminal unless the person first sees
   the exact text with hidden characters shown, confirms it, and the
   action is recorded as their own.
2. I cannot tell a real lane from a fabricated one: a public bundle is
   a projection the chain cannot be checked across, and the signing key
   is not tied to the person who opened the pull request. Draft: a
   foreign lane view MUST state that every event, verification level
   and witness mark in it is asserted by the publisher's key, and MUST
   show whether that key matches a signature on the pull request's
   commits in the local clone.

## Important

3. Nothing redacts on import, and private paths are not named. Drafts:
   import MUST verify the bundle's signature as received, then apply
   the importing tenant's SEC-08 rules before storing, and record both
   results; the publish rule set MUST redact absolute paths, usernames,
   hostnames and email addresses.
4. `parinn import https://…` is a network fetch that breaks B0 and I4;
   import must take a file or a ref already fetched.
5. The contributor's sharing path is unspecified. Draft: a lane bundle
   MUST be shareable as a plain file or a git ref, with no host,
   peering or account.
6. The traces disagree on the rule that matters most (trust by origin
   versus nothing signed is trusted), ids collide, and backward INJ-10
   puts sender-chosen names into TrustedText. Draft: an imported event
   whose claimed origin is a local origin MUST be refused and audited,
   and origin MUST come from the verified signing key alone.
7. The accepted contribution's link has no requirement. Draft: a landed
   commit MUST be linkable to the imported foreign lane behind it, with
   a proof class, and the bundle MUST be retained locally.
8. No reject action and no "why". Draft: a foreign lane MUST show
   PRV-07 flags and hidden characters in place, and rejecting or
   quarantining it MUST record the maintainer's stated reason.

## Minor

9. Adoption from a foreign lane MUST be per displayed text, never a
   whole lane or origin.
10. "From @kai" shows a contributor-supplied name against the petname
    rule.
11. RCL-08's lane scope must say whether a linked foreign origin is
    inside it; it should need its own explicit scope.
12. Public bundles leave prompts out by default, thinning the story;
    the withheld-count banner helps.

Verdict: the trust model is right, but it only serves my journeys once
findings 1–3 are fixed.
