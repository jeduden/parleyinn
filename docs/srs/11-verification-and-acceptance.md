---
title: "11. Verification and acceptance"
summary: >-
  The evaluation plan with baselines and acceptance targets, and the
  definition of done for v1.0.
---
# 11. Verification and acceptance

## 11.1 Evaluation plan

Evaluation runs in a separate module (`parinn-eval`) with versioned datasets
and fixed seeds so every result is reproducible. **All acceptance evaluations
MUST use the Claude models and Claude Code versions used in production**, with
at least three repetitions and reported confidence intervals. Published numbers
from other systems are reference points only.

**Baselines:** (base-0) plain Claude Code; (base-1) Claude Code with
lossless-claude/lcm; (C) ParleyInn full; (C−L) ParleyInn without landmarks;
(C−K) ParleyInn
without kernel.

| Area                    | Method                                                                                                                                                                                                                                                                                                                            | Acceptance target                                                                                                                       |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Recall after compaction | Long scripted runs with planted exact values (IDs, numbers, paths, error strings); query after 1, 3, 5, and 10 compactions                                                                                                                                                                                                        | ≥ 95%† exact recall at every depth                                                                                                      |
| Constraint survival     | ConstraintRot-style scenarios with constraints stated mid-run; count violations after N compactions                                                                                                                                                                                                                               | No increase over the same tasks without compaction                                                                                      |
| Poisoning resistance    | Six attack classes (explicit command insertion, conditional command insertion, salience-driven compaction poisoning, policy-conformant fact injection, false precedent insertion, skill-procedure insertion) delivered via web, tool output, files, file names, and MCP results; attack success measured in later, unrelated runs | Zero untrusted bytes in any restore block (hard); attack success lower than base-1, and any increase over base-0 triaged and documented |
| Flagging quality        | Labelled corpus of benign and malicious untrusted content                                                                                                                                                                                                                                                                         | Precision ≥ 0.9†; recall reported                                                                                                       |
| Long-history QA         | LongMemEval-S and BEAM (1M and 10M) subsets answered through ParleyInn tools                                                                                                                                                                                                                                                      | Reported with per-category breakdown                                                                                                    |
| Recall tool use         | Tasks answerable only from evicted history                                                                                                                                                                                                                                                                                        | The agent calls recall tools in ≥ 80%† of them                                                                                          |
| Cost                    | Identical tasks on base-0 and C                                                                                                                                                                                                                                                                                                   | Prompt-cache hit rate not lower than base-0 by more than 2 points; model-token delta reported                                           |
| Performance             | NFR-01..05 on synthetic 1M- and 10M-event stores                                                                                                                                                                                                                                                                                  | All targets met                                                                                                                         |
| Robustness              | Crash-consistency test, concurrency soak, malformed transcripts, pathological queries                                                                                                                                                                                                                                             | Zero corruption, zero lost events, zero hangs beyond deadlines                                                                          |
| Failure visibility      | Fault injection: disk full, permission errors, killed hook handlers, malformed events, schema mismatch                                                                                                                                                                                                                            | 100% of faults visible in counters, audit log, and `doctor` exit status                                                                 |
| Quarantine              | Quarantine planted events mid-run                                                                                                                                                                                                                                                                                                 | Never returned or rendered afterwards; record intact                                                                                    |
| Determinism             | `rebuild` on every evaluation store                                                                                                                                                                                                                                                                                               | Derived artifacts identical in content to before the rebuild in 100% of rebuilds                                                        |
| Multi-run reasoning     | Questions needing facts from many runs                                                                                                                                                                                                                                                                                            | Reported as a tracked known weakness; kernel-assisted vs plain recall compared                                                          |

## 11.2 Definition of done for v1.0

1. Every P0 requirement implemented and verified by its stated method; every P1
   implemented or deferred with signed-off rationale.
2. All assumptions in §2.3 verified or their dependent requirements re-planned.
3. Evaluation (§11.1) run and published in the repository, with all hard targets
   met.
4. External security review against §6 completed, with no open high or critical
   findings.
5. Two-week pilot on a production runner pool with zero data loss, zero
   agent-blocking incidents, and every non-zero failure counter explained.
6. Documentation complete (NFR-14).
