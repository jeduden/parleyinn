# Plans

## In progress

<?catalog
glob:
  - "plan/*.md"
  - "plan/*/plan.md"
  - "!plan/proto.md"
where: 'status: "🔳"'
sort: numeric:id
header: |

  | ID  | Model | Title |
  |-----|-------|-------|
row: "| {id} | {model} | [{title}]({filename}) |"
footer: |

empty: |

  Nothing in progress.

?>

| ID         | Model  | Title                                                                                                            |
| ---------- | ------ | ---------------------------------------------------------------------------------------------------------------- |
| 2609292011 | sonnet | [The remaining engineering gates in CI](plan/2609292011_ci-engineering-gates/plan.md)                            |
| 2609292012 | opus   | [M1: record and recall](plan/2609292012_m1-record-and-recall/plan.md)                                            |
| 2609292156 | sonnet | [The repository's knowledge follows ParleyInn's layers](plan/2609292156_knowledge-follows-parinn-layers/plan.md) |
<?/catalog?>

## All plans

<?catalog
glob:
  - "plan/*.md"
  - "plan/*/plan.md"
  - "!plan/proto.md"
sort: numeric:id
header: |

  | ID  | Status | Model | Title |
  |-----|--------|-------|-------|
row: "| {id} | {status} | {model} | [{title}]({filename}) |"
footer: |

empty: |

  No plans yet.

?>

| ID         | Status | Model  | Title                                                                                                                   |
| ---------- | ------ | ------ | ----------------------------------------------------------------------------------------------------------------------- |
| 2609292002 | ✅     | opus   | [Bootstrap the repository: SRS, requirement matrix, CI and release](plan/2609292002_bootstrap-repository.md)            |
| 2609292003 | ✅     | sonnet | [Spike S1: record the hook contract](plan/2609292003_spike-s1-hook-contract/plan.md)                                    |
| 2609292004 | ✅     | opus   | [Spike S2: the SQLite driver for the Rust core](plan/2609292004_spike-s2-sqlite-driver/plan.md)                         |
| 2609292005 | ✅     | sonnet | [Spike S3: catalogue the transcript formats](plan/2609292005_spike-s3-transcript-formats/plan.md)                       |
| 2609292006 | ✅     | sonnet | [Spike S4: prove plugin distribution](plan/2609292006_spike-s4-plugin-distribution/plan.md)                             |
| 2609292007 | ✅     | opus   | [Spike S5: Lua or SQL for the kernel](plan/2609292007_spike-s5-kernel-language/plan.md)                                 |
| 2609292008 | ✅     | sonnet | [Spike S6: can compaction be deferred](plan/2609292008_spike-s6-compaction-control/plan.md)                             |
| 2609292009 | ✅     | sonnet | [Spike S7: the Rust MCP SDK or an in-house server](plan/2609292009_spike-s7-mcp-sdk/plan.md)                            |
| 2609292010 | ✅     | sonnet | [Spike S8: calibrate the token estimator and freeze the targets](plan/2609292010_spike-s8-token-estimator/plan.md)      |
| 2609292011 | 🔳     | sonnet | [The remaining engineering gates in CI](plan/2609292011_ci-engineering-gates/plan.md)                                   |
| 2609292012 | 🔳     | opus   | [M1: record and recall](plan/2609292012_m1-record-and-recall/plan.md)                                                   |
| 2609292013 | 🔲     | opus   | [M2: pins and restore](plan/2609292013_m2-pins-and-restore.md)                                                          |
| 2609292014 | 🔲     | opus   | [M3: landmarks and lifecycle](plan/2609292014_m3-landmarks-and-lifecycle.md)                                            |
| 2609292015 | 🔲     | opus   | [M4: the hermetic compute kernel](plan/2609292015_m4-compute-kernel.md)                                                 |
| 2609292016 | 🔲     | opus   | [M5: hardening and evaluation](plan/2609292016_m5-hardening-and-evaluation.md)                                          |
| 2609292156 | 🔳     | sonnet | [The repository's knowledge follows ParleyInn's layers](plan/2609292156_knowledge-follows-parinn-layers/plan.md)        |
| 2609301942 | ✅     | sonnet | [Agent review through the reviewer app](plan/2609301942_agent-review-app/plan.md)                                       |
| 2610012322 | ✅     | opus   | [Scope ParleyInn for agent fleets: shared, real-time, public sessions](plan/2610012322_parinn-for-agent-fleets/plan.md) |
| 2610022338 | 🔲     | opus   | [ParleyInn's lane experience and peer network: no central service](plan/2610022338_parinn-network-side/plan.md)         |
| 2610031222 | 🔲     | opus   | [Sandboxed agents: the boundary ParleyInn's owner acts rest on](plan/2610031222_agent-sandboxing/plan.md)               |
| 2610050706 | ✅     | sonnet | [Move the whole toolchain from Go to Rust](plan/2610050706_toolchain-to-rust/plan.md)                                   |
| 2610112057 | 🔲     | opus   | [M7: standalone room view](plan/2610112057_m7-standalone-room-view.md)                                                  |
| 2610112058 | 🔲     | opus   | [M8: peer network](plan/2610112058_m8-peer-network.md)                                                                  |
| 2610112059 | 🔲     | opus   | [M9: review, publishing, bridges](plan/2610112059_m9-review-publishing-bridges.md)                                      |
| 2610112203 | ✅     | sonnet | [Spike S13: can a hook tell which run compacted](plan/2610112203_spike-s13-compaction-ambiguity/plan.md)                |
| 2610190944 | ✅     | sonnet | [Ship third-party license texts inside every executable](plan/2610190944_third-party-licenses/plan.md)                  |
<?/catalog?>
