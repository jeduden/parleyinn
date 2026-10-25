# Round 7

Three blind lens reviewers (`result.json`) reported 9 needs-fix findings,
18 minor ones and 2 of severity `later`. No needs-fix finding repeated a
closed theme; each formed a new theme, closed as follows (`verify.json`).

| Theme | Closed as                                                                 |
| ----- | ------------------------------------------------------------------------- |
| DM-DL | deferred: OQ-47, M8 (`later`: retiring a writer before PRV-10)            |
| DM-DM | deferred: OQ-47, M7 (a kept cut act and a lapsed rule-level loosening)    |
| DM-DN | fixed: a nested harness session's input is `harness_text` (PRV-08)        |
| DM-DO | fixed: ParleyInn detects pin candidates only in trusted `user` events     |
| DM-DP | fixed: the model defines `harness_text` (PRV-08)                          |
| DM-DQ | fixed: a quarantine or purge resets kernel variables (CMP-07)             |
| DM-DR | fixed: the model carries ADM-04's full set of held changes                |
| DM-DS | fixed: within each group of rooms, intents fill the budget first (PIN-08) |
| DM-DT | deferred: OQ-49, M5 (a confirmed candidate's room; shown meanwhile)       |
| DM-DU | fixed: before M7 a join request without its principal's ask is refused    |

The stakeholder stopped the loop after this round to re-evaluate the
process, so no regression pass ran over these edits.

## Count

Open themes stay at 0: the round opened 10 and closed 10.
