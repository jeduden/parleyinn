---
title: "3. Rationale"
summary: >-
  The research findings behind the design — lossless record, verbatim
  pins, pull-only recall, code access, cache stability, and lessons
  from lcm.
---
# 3. Rationale

This section summarizes the research that shaped the requirements. Appendix A
maps every individual concern to requirements.

1. **Keep the record separate from the view.** The strongest published systems
   (Scroll, LCM, CWL) never let the prompt be the only copy of history. In
   Scroll's ablation, replacing history with summaries at ingestion dropped the
   BEAM-10M score from 73.1 to 19.9. → ParleyInn is lossless and event-sourced
   (I1,
   I10).
2. **Summarization erases constraints.** *Governance Decay*: constraint
   violations rose from 0% to 30% after compaction, up to 59% for some models.
   *Compaction Cliff*: Claude Code's `/compact` on Sonnet 4.6 kept 53% of safety
   rules after one round and 10% after five. *Constraint Pinning* restored 0%
   violations at about 47 model tokens per re-injection. → Verbatim pins (I3).
3. **Automatic injection multiplies memory poisoning.** In a cross-run
   poisoning study, a design that injected memory at each agent run's start
   carried poisoned entries into later runs 64.7% of the time versus 17.4% for
   pull-only retrieval. Store size does not dilute poison: AgentPoison reached
   ≥80% attack success at <0.1% poison rate. → Pull-only recall (I2), quarantine
   (I5).
4. **Code access to history beats serializing it.** Scroll without its
   persistent REPL lost 7.3 points, concentrated in tasks that combine evidence
   from many parts of a history. → Kernel (§5.8).
5. **Prompt-cache stability matters.** TokenPilot showed that context mutations
   shifting the prompt prefix break caching; stabilizing the prefix cut costs by
   56–87%. → Deterministic injection (INJ-06).
6. **Operational lessons from existing tools.** lossless-claude/lcm: a shared
   loopback daemon on a fixed port allowed cross-process interference (#563);
   one unbounded FTS5 query froze the daemon for minutes (#596); missing auth
   headers silently dropped 21,279 events (#292); `doctor` rewrites
   `settings.json` with no read-only mode. → Daemonless design, bounded queries,
   no-silent-failure rules, explicit harness configuration changes (I6, I7, I9).
7. **Published results are not controlled comparisons.** Memory benchmarks use
   different reader models and setups, and most context-management results come
   from non-Claude models. → All acceptance criteria are measured on our own
   Claude models and workloads (§11).
