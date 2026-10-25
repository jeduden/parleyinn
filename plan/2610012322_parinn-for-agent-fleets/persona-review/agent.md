# Persona review: Claude (the agent)

Target: pitch v7, plan 2610012322 (plan, traces, ux/) and plan
2610022338, at 618fafb.

## Blocking

1. Steering puts owner-voice text into my context outside TrustedText,
   and it can carry untrusted content: late approvals quote
   agent-chosen commands, and deny reasons go out through a hook.
   UI-04 takes the safe route; the UX contradicts it. Draft: owner
   steering, approvals and replies MUST reach an agent only through the
   harness's own input interface, never through ParleyInn hook output, and
   MUST NOT embed any record content.
2. The owner's other devices contradict the trust policy: the UX treats
   their events as operator, the traces make every imported event
   untrusted, so pins and instructions go missing or the policy breaks
   quietly. Draft: a pin trusted on one of the owner's nodes but not
   injected on another MUST be counted in that node's restore block,
   with the reason.

## Important

3. Knowing messages are waiting has three conflicting answers, and the
   requirement draft has none. Draft: a waiting-message notice MUST be
   TrustedText made only of a count, local petnames and recall
   addresses, MUST be audited, and MUST NOT start a turn.
4. The recall envelope does not say where an item came from. Draft:
   every recalled item MUST carry its origin, actor, trust, whether it
   was imported, and its chain status.
5. Re-keying the project by clone widens which pins come back. Draft:
   each pin MUST be scoped to a lane or explicitly to the whole
   project, and a restore block MUST hold only its own lane's pins and
   project-wide pins.
6. Hook holds break flow and the hook budgets. Draft: a held permission
   hook MUST have its own NFR-01 budget, and an expired hold MUST reach
   the agent as fixed "held" text, never as a rejection of the action.
7. A watchdog nudge writes text to the agent with no owner act. Draft:
   ParleyInn MUST NOT send text to an agent except on an owner act recorded
   at that time.

## Minor

8. The same owner act gets three different classes (user, operator).
9. The "one address scheme" is not one scheme. Draft: every id ParleyInn
   shows an agent MUST be accepted by get or expand.
10. The two traces disagree on recall scope (session versus lane).
11. When steering ships is unclear (P2/M9 versus phase 1).

Verdict: lane-scoped recall, pull-only and pins in the pitch serve me,
but as written the steering and multi-device designs would give me
untrusted text in the owner's voice and lose pins between nodes.
