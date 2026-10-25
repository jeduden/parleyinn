Round 33 standing review of docs/srs/ and features/*.feature against components-and-surfaces.md

**Result:** I found 9 defects. 6 need a fix and 3 need a ruling from the stakeholder. All of them are naming slips or gaps around the component and surface boundary. None needs new invariant wording.

**Clean:**

- The Component column of §6.3 holds the same six components as the model: core, room-view component, launcher, peer, publish and bridge component.
- Every id my file cites exists and says what my file says: I4, OQ-32, §6.3, ADM-04, SEC-01, LANE-28, SEC-20, VIEW-03, OWN-16, CMP-05, VIEW-14, VIEW-08, VIEW-05, I6, I7, SEC-23, VIEW-22, §9.2.
- I4, I6 and I7 read the same in invariants.md, my file, SEC-01, SEC-19, SEC-29, ENG-12, ENG-16, VIEW-18 and SEC-23.
- The surface names are used consistently across the SRS and the scenarios: Fleet, Room page, Catch up, Needs you, Health with its Setup part, Peers, context lens, quarantine list, forensic view, outcome window and harness strip.
- None of the excluded terms appears: "run component", "lane view", "trusted boundary".
- "Boundary" is always qualified when it is not a network boundary ("memory boundary", "span boundary kind", "crate boundaries").

**Findings, most important first:**

1. **`parinn ui --phone` (question).**

  - My file says the CLI is `parinn` "but for `parinn ui`". It also says the room-view component records only "the principal acts taken in it", that is, in the browser room view.
  - Yet OWN-16, §9.5 row 80, §4.2 and the OWN-16 scenario make `parinn ui --phone` a widening principal act taken at a terminal, citing OWN-12 ("every CLI verb").
  - OWN-02 accepts a terminal act only from "the CLI or TUI", and refuses an act from anywhere else. trust-and-flow.md's Principal surface says the same.
  - So no principal surface or component in the model covers this act. OQ-39 and OQ-40 sit next to it, but neither decides it.
  - The hub's Names section also counts `ui` and `launch` among the CLI's commands. "CLI" therefore carries two meanings.

2. **OQ-40's scope (question).** OQ-40 and the §6.3 note list only the room-view component, the launcher and the bridge component (rows 8, 10, 20 and 21). But SEC-25 has the peer component record each refused segment as REC-21's structural event, in a writer the core seals. SEC-10's open mark already says "another component", so the gap is only in OQ-40's wording. record.md's recorder list also omits the peer component; that belongs to the record specialist.

3. **CON-03 "webview" (question).** CON-03 lets the browser room view client run "in a webview". My file's closed list of clients names only the browser, served through the room-view component. No row in §6.3 says how a webview reaches the record. The wording runs ahead of ADR-2610042341, which is still proposed.

4. **security.feature:242 (needs-fix).** The SEC-20 scenario title says "the room view binds to loopback". Binding is the room-view component's job.

5. **security.feature:276 (needs-fix).** The SEC-22 example "every boundary disabled" would include B0. Managed policy can disable only B1, B2 and B3.

6. **administration.feature:47–64 (needs-fix).** The ADM-04 outline's `component` column holds "parinn status", which is not a component.

7. **"UI" used as a name for the room view (needs-fix, three places).**

  - 06-security.md:56, threat T15
  - 09b-lane-vocabulary.md:106–107
  - index.md:15, the Implementation language row

8. **12-delivery-plan.md:50 (needs-fix).** It writes "catch-up" where the surface is named "Catch up".

```json
[
{"file":"docs/srs/05c-principal-and-peer-requirements.md","line":32,"quote":"whose issue (`parinn ui --phone`) MUST be a widening principal act at a terminal (OWN-12)","term":"CLI / Room-view component / principal surface","breaks":"Core entry (components-and-surfaces.md:12-13) says the CLI is `parinn` 'but for `parinn ui`', and Room-view component (:31-33) records only 'the principal acts taken in it' (the browser room view). The `--phone` act runs as the room-view component's entry point at a terminal. OWN-02 (05c:18, 'the CLI or TUI at a terminal under OWN-12') and trust-and-flow.md:40-41 accept terminal acts only from the CLI or TUI, and OWN-02 refuses an act from anywhere else. Same conflict in 09a-command-line-interface.md:80, 04-reference-architecture.md:65-67, 06b-boundary-register.md:39 (row 8 'records the browser room view's principal acts') and features/principal-acts.feature:204. The hub's Names section (index.md:108-111) also counts `ui`/`launch` among CLI commands, giving CLI two meanings","fix":"Stakeholder picks one. (a) In components-and-surfaces.md Room-view component add 'and, at the terminal that starts it, the widening act issuing a phone-scoped room-view secret (`parinn ui --phone`, OWN-16)'; add that terminal entry point to OWN-02's surface list and to trust-and-flow.md's Principal surface (trust-and-flow specialist); add it to §6.3 row 8 Reach (signing under OQ-40). Or (b) move the issue into a core CLI verb that `parinn ui --phone` then reads from the record, changing OWN-16, §9.5 and §4.2. Either way qualify the hub's 'CLI command' so `ui`/`launch` are commands, not the core's CLI","kind":"question","invariant":false,"hard_to_revert":true},
{"file":"docs/srs/13-open-questions-and-risks.md","line":47,"quote":"Who signs and seals what the room-view component, the launcher and the bridge component record?","term":"Component (peer component, publish component)","breaks":"SEC-25 (06-security.md:115: 'MUST refuse and audit a segment failing either, recording it as REC-21 says') has the peer component record a structural event in this node's device seat writer, which REC-19/SEC-10 leave to the core to seal. The git carrier in the publish component does the same through PRV-09 and ADM-15. OQ-40's question and the §6.3 note (06b-boundary-register.md:25-26, 'Rows 8, 10, 20 and 21 rest on who signs and seals what their component records') leave these out, while SEC-10's mark ('another component') covers them","fix":"Widen OQ-40's question to 'the room-view component, the launcher, and the peer, publish and bridge components', and the §6.3 note to name rows 12 and 19 too. Or state in SEC-25/REC-21 that the core records the refusal. record.md's recorder list (record.md:87-91) lacks the peer component: record specialist","kind":"question","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/02-context.md","line":77,"quote":"The browser room view client's TypeScript runs in a webview or a browser, never in a Node.js runtime.","term":"Room view (its closed set of clients)","breaks":"Room view lists the browser served through the room-view component, plus the reduced clients. A webview client is in neither the model nor §6.3, and nothing says whether it reaches the record through the room-view component on loopback under SEC-20. This runs ahead of the still-proposed ADR-2610042341 (IPC backend)","fix":"Either drop 'a webview or' from CON-03 until that ADR's SRS changes land, or add to components-and-surfaces.md Room view: 'a webview counts as the browser only when it loads the browser room view from the room-view component on loopback under SEC-20'","kind":"question","invariant":false,"hard_to_revert":false},
{"file":"features/security.feature","line":242,"quote":"Scenario: the room view binds to loopback and accepts only its own launch secret","term":"Room view vs Room-view component","breaks":"Room view is a client of the record; listening on loopback and minting the launch secret belong to the Room-view component (SEC-20: 'The room-view component MUST listen only on a loopback network address'). The steps already say 'the room-view component'","fix":"features/security.feature:242 -> 'Scenario: the room-view component binds to loopback and accepts only its own launch secret'","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"features/security.feature","line":276,"quote":"| every boundary disabled                     | the person turns on the peer component","term":"Boundary","breaks":"Boundary is one of B0 core, B1, B2 and B3, so 'every boundary' includes B0. SEC-22 and I4 let managed policy disable only B1, B2, B3 and the launcher","fix":"features/security.feature:276 -> '| B1, B2 and B3 disabled |'","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"features/administration.feature","line":"47-64","quote":"When an agent runs and the person starts \"<component>\" ... | the person's config.toml containing \"recal.max_k = 10\" | parinn status |","term":"Component","breaks":"Component is a closed set (core, room-view component, launcher, peer, publish, bridge). 'parinn status' is a CLI command inside the core, not a component, yet the outline's `component` column holds it","fix":"features/administration.feature: rename the column and placeholder `<component>` to `<started>` (or 'component or command') at lines 47, 48 and 52","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/06-security.md","line":56,"quote":"| T15 | Local web attack on the UI ","term":"Room view (browser room view)","breaks":"'UI' is a second name for the browser room view and its room-view component; each concept has one name (hub)","fix":"06-security.md:56 -> 'Local web attack on the browser room view'","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/09b-lane-vocabulary.md","line":"106-107","quote":"The UI never says \"secure\".","term":"Room view / surface","breaks":"'UI' stands in for the room view's surfaces and reduced clients, which the model names","fix":"09b-lane-vocabulary.md:106-107 -> 'No surface or reduced client says \"secure\".'","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/index.md","line":15,"quote":"Rust; TypeScript for the UI page","term":"Room view (browser room view client)","breaks":"'the UI page' names the browser room view client, which CON-01 and ENG-02 call 'the browser room view client'","fix":"index.md:15 -> 'Rust; TypeScript for the browser room view client'","kind":"needs-fix","invariant":false,"hard_to_revert":false},
{"file":"docs/srs/12-delivery-plan.md","line":50,"quote":"the recorded-weekend fixture passes catch-up, search-to-replay and room verify","term":"Catch up","breaks":"The surface is 'Catch up'; the kebab form belongs only to the CLI command (`parinn catch-up show`), per the hub's naming rule","fix":"12-delivery-plan.md:50 -> 'passes Catch up, search-to-replay and room verify'","kind":"needs-fix","invariant":false,"hard_to_revert":false}
]
```

**Instruction feedback:**

- The hub's Names section counts `ui` and `launch` as CLI commands, while my file says "the CLI" excludes them. The instructions don't say whether my file or the hub (the domain-model agent) owns the meaning of "CLI".
- Several findings span my file and another specialist's file (for example, trust-and-flow's Principal surface). The instructions don't say whether to file those as "question" or "other-specialist".
- The JSON `file` field takes a single path, but "group repeats" pulls the other way. I split the three "UI" uses into separate entries.
- It is unclear whether proposed ADRs may be read for context. I read ADR-2610042341 to judge the CON-03 finding; plan/ and research/ were not opened.
