# Part 2: fix clusters. Uses c(), A, S, M, F from part 1.

def fx(title, rids, owners, locations, problem, fix, touches, htr=False, conflict=""):
    A(c(title, rids, owners, locations, problem, fix, "fix", conflict, [], htr, touches))

fx("LANE-01 sends every event to a run seat or personal-room seat, missing REC-19's seat that ingest starts",
   ["hub#1", "whole#3"], ["hub", "record"],
   [S+"05b-lane-requirements.md:14", M+"index.md:70-72", S+"05-functional-requirements.md:34"],
   "Hub relation 6 and REC-19 send what `parinn ingest` appends for a witnessed run to the seat ingest starts beside that run seat. LANE-01's 'Each event MUST be written to exactly one seat's writer: ... else to its personal-room seat, never refused' has no such exception, so its MUST contradicts REC-19 for ingested appends.",
   "LANE-01: after 'else to its personal-room seat, never refused' add '; what `parinn ingest` appends for a witnessed run goes instead to the seat ingest starts beside that run seat (REC-19)'.",
   [S+"05b-lane-requirements.md"], htr=True)

fx("LANE-15 treats every foreign room as one read from a bundle",
   ["places#1", "whole#1"], ["places"],
   [S+"05b-lane-requirements.md:27"],
   "Foreign room also covers a room every seat of its principal has left or lost to a kick or a bar, which PIN-10's and RCL-10's scenarios and the hub rely on. Such a room has no bundle's principal key and no pull request, so LANE-15's MUSTs cannot be met for it.",
   "LANE-15: 'A foreign room MUST open in the Room page, marked foreign; one imported from a bundle MUST state that every event, evidence class and proof class in it is asserted by the bundle's principal key, and MUST show whether ...'.",
   [S+"05b-lane-requirements.md"],
   conflict="Wording only: places#1 says 'one read from a bundle', whole#1 'one imported from a bundle'; the latter matches the hub ('Only a bundle is imported') and Foreign room ('the room of an imported bundle').")

fx("A provenance scenario speaks of 'unsigned' trusted sources",
   ["trust-and-flow#3", "whole#4"], ["trust-and-flow", "record"],
   [F+"provenance.feature:37"],
   "Trusted sources has no 'unsigned' subset, and 'unsigned' (Seal, §9.7.5) means events after the newest seal; this node's operator, structural, harness_meta and user events are sealed. Here it means 'signed by no device key', a second sense.",
   "Scenario title: 'the default trust policy trusts this node's own trusted sources only on this node, ...'.",
   [F+"provenance.feature"])

fx("§3 points the kernel at §5.8; it is §5.7",
   ["pins-and-context#13", "principals-and-agents#9"], ["record"],
   [S+"03-rationale.md:31"],
   "The Compute kernel is §5.7 (CMP, 05-functional-requirements.md:116); §5.8 is Administration.",
   "03-rationale.md:31: '→ Kernel (§5.7).'",
   [S+"03-rationale.md"])

fx("'Concurrent' is used for things at the same time in NFRs, OWN-23, ENG-09, §12 and scenarios",
   ["acts-and-roles#4"], ["acts-and-roles"],
   [S+"07-non-functional-requirements.md:19", S+"07-non-functional-requirements.md:22", S+"07-non-functional-requirements.md:23", S+"05c-principal-and-peer-requirements.md:39", S+"10-engineering-quality.md:29", S+"12-delivery-plan.md:48", F+"non-functional.feature:59", F+"non-functional.feature:62", F+"non-functional.feature:92", F+"non-functional.feature:95", F+"engineering.feature:90", F+"record.feature:70"],
   "The model defines Concurrent only for acts and events, in the causal sense, and the hub gives every word one meaning. These places use it for runs, appends, instances, delegates and writers at the same time.",
   "Use 'at once' or 'in parallel': '≥ 50 runs writing at once', 'Appends running in parallel MUST never corrupt ...', 'ten instances at once', OWN-23 'the maximum delegates at once', ENG-09 '50 writers appended in parallel', §12 'many agents at once'; the scenarios likewise.",
   [S+"07-non-functional-requirements.md", S+"05c-principal-and-peer-requirements.md", S+"10-engineering-quality.md", S+"12-delivery-plan.md", F+"non-functional.feature", F+"engineering.feature", F+"record.feature"])

fx("LANE-16 and LANE-17 ask the role of every seat, but the owner's device seats have none",
   ["acts-and-roles#5"], ["acts-and-roles", "principals-and-agents"],
   [S+"05b-lane-requirements.md:29", S+"05b-lane-requirements.md:28", F+"lane.feature:260"],
   "The owner's device seats stand beside the roles, and a paired phone's personal-room seat has none. LANE-17 shows 'the role of each of its seats' and LANE-16 'each seat its role and capabilities'; LANE-22 already says 'role, or owner'.",
   "LANE-17: 'the role of each of its seats, or owner'; LANE-16: 'The room view MUST show each seat its role, or owner, and its capabilities'; lane.feature:260 to match.",
   [S+"05b-lane-requirements.md", F+"lane.feature"])

fx("Room merge's list of what room state covers omits branch links",
   ["acts-and-roles#6"], ["acts-and-roles", "git-and-forge"],
   [M+"acts-and-roles.md:181-184", M+"acts-and-roles.md:191-193"],
   "The Room merge entry resolves branch links ('Of two branch links naming one branch, the first in causal order stands') and LANE-01 sends branch links to LANE-31, but the coverage list, which reads as closed, leaves them out.",
   "acts-and-roles.md Room merge: '... bars, branch links, handovers and their offers, ...'.",
   [M+"acts-and-roles.md"])

fx("Capability is defined as what a role grants, but ownership and appointments grant capabilities too",
   ["acts-and-roles#8"], ["acts-and-roles"],
   [M+"acts-and-roles.md:127"],
   "The owner's device seats have every room capability without a role (Owner), and writing a room summary comes only from the facilitator's appointment.",
   "acts-and-roles.md: 'Capability: What a role, an appointment or ownership lets a seat do.'",
   [M+"acts-and-roles.md"])

fx("A security scenario title gives the room view the room-view component's job",
   ["components-and-surfaces#4"], ["components-and-surfaces"],
   [F+"security.feature:242"],
   "Room view is a client of the record; listening on loopback and minting the launch secret belong to the room-view component (SEC-20). The steps already say 'the room-view component'.",
   "security.feature:242: 'Scenario: the room-view component binds to loopback and accepts only its own launch secret'.",
   [F+"security.feature"])

fx("'every boundary disabled' includes B0, which managed policy cannot disable",
   ["components-and-surfaces#5"], ["components-and-surfaces"],
   [F+"security.feature:276"],
   "Boundary is one of B0 to B3; SEC-22 and I4 let managed policy disable only B1, B2, B3 and the launcher.",
   "security.feature:276: '| B1, B2 and B3 disabled |'.",
   [F+"security.feature"])

fx("An administration outline's `component` column holds a CLI command",
   ["components-and-surfaces#6"], ["components-and-surfaces"],
   [F+"administration.feature:47", F+"administration.feature:48", F+"administration.feature:52"],
   "Component is a closed set (core, room-view component, launcher, peer, publish, bridge); `parinn status` is a CLI command inside the core.",
   "Rename the column and placeholder `<component>` to `<started>` (or 'component or command') at lines 47, 48 and 52.",
   [F+"administration.feature"])

fx("'UI' is a second name for the browser room view and its surfaces",
   ["components-and-surfaces#7", "components-and-surfaces#8", "components-and-surfaces#9"], ["components-and-surfaces", "record"],
   [S+"06-security.md:56", S+"09b-lane-vocabulary.md:106-107", S+"index.md:15", M+"record.md:156-157"],
   "Each concept has one name, and the model names the browser room view client, the room view's surfaces and the reduced clients. T15, §9.7.5's 'The UI never says \"secure\"' and the SRS metadata table call them 'the UI'. While bundling, the same sentence was found in record.md:156-157 (Integrity status); the hub's own 'hide' row also says 'say the UI does not show it'.",
   "06-security.md:56 T15: 'Local web attack on the browser room view'; 09b:106-107 and record.md:156-157: 'No surface or reduced client says \"secure\".'; docs/srs/index.md:15: 'Rust; TypeScript for the browser room view client'. Optionally the hub's 'hide' row: 'say no surface shows it'. Leave the SRS change log (index.md:31) alone: it is a historical record.",
   [S+"06-security.md", S+"09b-lane-vocabulary.md", S+"index.md", M+"record.md"])

fx("§12 says 'catch-up' for the Catch up surface",
   ["components-and-surfaces#10"], ["components-and-surfaces"],
   [S+"12-delivery-plan.md:50"],
   "The surface is 'Catch up'; the kebab form belongs only to the CLI command (`parinn catch-up show`), per the hub's naming rule.",
   "12-delivery-plan.md:50: '... passes Catch up, search-to-replay and room verify'.",
   [S+"12-delivery-plan.md"])

fx("The model's tree is a commit's snapshot, which excludes every own check's uncommitted tree",
   ["git-and-forge#2"], ["git-and-forge"],
   [M+"git-and-forge.md:30-31", M+"git-and-forge.md:38-39", S+"05b-lane-requirements.md:18"],
   "Own check runs 'on the latest worktree checkpoint plus the recorded edits' (LANE-05: 'the tree is the latest worktree checkpoint plus the edits recorded since'), which is no commit's snapshot. Read literally, Result excludes every own check.",
   "Model: '**tree** (git's snapshot of a set of files: a commit's, or a worktree's latest worktree checkpoint plus the edits recorded since)'.",
   [M+"git-and-forge.md"])

fx("LANE-20 does not say a criterion naming a check's command makes that check expected",
   ["git-and-forge#3"], ["git-and-forge"],
   [S+"05b-lane-requirements.md:32", M+"git-and-forge.md:26-27", F+"lane.feature:293"],
   "git-and-forge.md cites LANE-20 for 'A check is expected when one of the intent's criteria names its command', but LANE-20 only says 'a criterion MAY name a check's command'. Only the scenario asserts the rest, so the Pending and Absent check states have no normative trigger.",
   "LANE-20: 'a criterion MAY name a check's command, which makes that check expected on each branch the room names'.",
   [S+"05b-lane-requirements.md"])

fx("A forge scenario title gives every branch of the room a pull-request link",
   ["git-and-forge#5"], ["git-and-forge"],
   [F+"lane.feature:111"],
   "A pull-request link connects a branch to its pull request; LANE-08 and the scenario's own steps (114, 117) give one only to the branch that has a pull request.",
   "lane.feature:111: '... and a branch with a pull request carries a pull-request link to it'.",
   [F+"lane.feature"])

fx("Proof classes are attached to rooms and results instead of landing links",
   ["git-and-forge#6", "git-and-forge#13"], ["git-and-forge", "hub"],
   [F+"lane.feature:90", F+"lane.feature:95", M+"git-and-forge.md:5", M+"index.md:49"],
   "A proof class is 'Of a landing link' (LANE-06). A LANE-06 scenario gives the proof class to the room, and git-and-forge.md's summary, repeated in the hub catalog, gives it to results.",
   "lane.feature:90 title: 'each room behind a landed commit is linked to it by a landing link with one proof class'; :95 'Then the room is listed through a landing link with proof class \"<proof>\"'. git-and-forge.md front matter summary: 'results with their evidence classes, landing links with their proof classes'; then `mdsmith fix` regenerates the hub catalog.",
   [F+"lane.feature", M+"git-and-forge.md", M+"index.md"])

fx("'Check' is used for LANE-16's role check, a refusal order and CI static analysis",
   ["git-and-forge#7"], ["git-and-forge"],
   [S+"05b-lane-requirements.md:38", F+"principal-acts.feature:158", S+"06-security.md:97", F+"security.feature:87-90"],
   "Check means a command executed on a tree. LANE-26 'refused by the check of LANE-16', the OWN-12 scenario 'refuses before any other check' and SEC-07 'a static check in CI' (and its scenario's 'the check fails') give it second meanings; the hub already renamed 'presence check' for the same reason.",
   "LANE-26: 'MUST be refused under LANE-16 and audited'; OWN-12 scenario: 'refuses before anything else is checked'; SEC-07 and its scenario: 'CI static analysis ... fails'.",
   [S+"05b-lane-requirements.md", F+"principal-acts.feature", S+"06-security.md", F+"security.feature"])

fx("'Evidence' is used for build-time reach analysis and for receipts' tamper evidence",
   ["git-and-forge#8", "git-and-forge#14"], ["git-and-forge", "record"],
   [S+"06-security.md:91", S+"06-security.md:109", S+"10-engineering-quality.md:41", F+"security.feature:10-14", F+"security.feature:236", F+"security.feature:362", F+"engineering.feature:142", M+"record.md:160"],
   "Evidence means the checks, CI attestations or text a result rests on. SEC-01, SEC-19, ENG-12 and their scenarios use it, unqualified ('that evidence'), for build-time reach analysis, and record.md's Receipt calls a head receipt 'the tamper evidence of VIEW-10 and SEC-27', a second meaning inside the model itself.",
   "SEC-01, SEC-19, ENG-12 and their scenarios: 'build-time reach analysis' / 'that analysis'. record.md:160: 'a head receipt lists every writer's chain head at a moment, which shows tampering under VIEW-10 and SEC-27'.",
   [S+"06-security.md", S+"10-engineering-quality.md", F+"security.feature", F+"engineering.feature", M+"record.md"])

fx("NFR-15 'rebuilds a worktree', which ParleyInn never writes",
   ["git-and-forge#9"], ["git-and-forge", "record"],
   [S+"07-non-functional-requirements.md:29", F+"non-functional.feature:166"],
   "A worktree is a git working tree on a node, which ADM-13 bars ParleyInn from writing, and 'rebuild' is `parinn rebuild`'s word for derived artifacts. VIEW-12 reconstructs the worktree's state from the record.",
   "NFR-15 and its scenario: 'reconstructs the worktree at an event within 500 ms† (VIEW-12)'.",
   [S+"07-non-functional-requirements.md", F+"non-functional.feature"])

fx("The keymap's 'compare versions' gives the comparison's word to intent-version diffs",
   ["git-and-forge#10"], ["git-and-forge"],
   [S+"09b-lane-vocabulary.md:164"],
   "'Compare' belongs to the comparison of two branches (VIEW-13). This key shows intent-version diffs (VIEW-21).",
   "09b keymap: '`V` / `s` | intent version diffs / since my verdict'.",
   [S+"09b-lane-vocabulary.md"])

fx("§2 names the evidence classes loosely",
   ["git-and-forge#11"], ["git-and-forge"],
   [S+"02-context.md:97"],
   "'claim versus own check versus CI' says 'CI' instead of the class name `CI attested` and leaves out `witness check`.",
   "02-context.md:97: 'claim versus own check, witness check and CI attested'.",
   [S+"02-context.md"])

fx("Plain-English 'evidence', 'result', 'comparison', 'check' and 'tree' in non-normative prose and an assumptions scenario",
   ["git-and-forge#12"], ["git-and-forge"],
   [S+"03-rationale.md:30", S+"03-rationale.md:41-42", S+"02-context.md:47", S+"04-reference-architecture.md:154", S+"04-reference-architecture.md:158", S+"11-verification-and-acceptance.md:12", S+"appendix-a-concern-traceability.md:18", F+"assumptions.feature:41-42", F+"assumptions.feature:60", F+"assumptions.feature:150-151"],
   "These model words appear in their ordinary sense in rationale, architecture, verification and an assumptions scenario. The hub gives every word one meaning (index.md:13); the reviewer asks for a rule on how strictly to treat such prose (see instruction edits).",
   "'facts from many parts', 'Published scores are not controlled experiments ... most context-management scores', 'Support so far', 'Poisoning findings', 'WAL tests', 'every measurement is reproducible', 'permission tests', '\"~/.claude/projects\" directory ... walks the directory'.",
   [S+"03-rationale.md", S+"02-context.md", S+"04-reference-architecture.md", S+"11-verification-and-acceptance.md", S+"appendix-a-concern-traceability.md", F+"assumptions.feature"])

fx("Turn reads as if each tool result starts a new turn",
   ["harness-facts#3"], ["harness-facts"],
   [M+"harness-facts.md:31-32", S+"05b-lane-requirements.md:26"],
   "'One exchange ... from an input to the model reply that ends it' lets each tool result start a turn, which LANE-14 forbids ('Nothing else MAY trigger a turn'). OWN-15's mid-turn steer, §9.7.1 Working/Idle and §9.4's 'turns 31-33 · Edit×6 Bash×4' assume a turn spans the tool loop.",
   "Model: 'One exchange between the harness and the model, from the input that starts it, through every tool call and tool result within it, to the model reply after which the harness waits for new input; a tool result never starts a turn.'",
   [M+"harness-facts.md"])

fx("LANE-14 requires a delegation grant for every delegation, so subagents' turns are forbidden",
   ["harness-facts#4"], ["harness-facts", "trust-and-flow"],
   [S+"05b-lane-requirements.md:26"],
   "A subagent's turn is triggered by its delegation, which has no delegation grant (OWN-23); OWN-24 already says 'its delegation grant when one applies'.",
   "LANE-14: 'or a delegation and its delegation grant, when one applies'.",
   [S+"05b-lane-requirements.md"])

fx("LANE-14 is cited for the harness_text classification, which is PRV-08",
   ["harness-facts#5", "harness-facts#6"], ["harness-facts"],
   [M+"harness-facts.md:37-38", S+"09b-lane-vocabulary.md:114-115"],
   "LANE-14 only says ingest 'MUST mark untrusted'; recording the line as `harness_text`, never a `user` event, is PRV-08.",
   "harness-facts.md:37-38 and §9.7.6 (09b:114-115): '(LANE-14, PRV-08)'.",
   [M+"harness-facts.md", S+"09b-lane-vocabulary.md"])

fx("§6.3 row 24 omits recalled content from what the harness sends its provider",
   ["harness-facts#10"], ["components-and-surfaces", "harness-facts"],
   [S+"06b-boundary-register.md:55"],
   "I4, §2 and VIEW-18 say the harness sends its model both recalled content an agent receives through a tool call and what ParleyInn writes through I2's closed paths; row 24 names only the latter.",
   "Row 24: 'carrying the recalled content agents receive through tool calls and what I2's closed paths write (I4)'.",
   [S+"06b-boundary-register.md"])

fx("RCL-07's scenario counts recalls with a 'counter'",
   ["hub#3"], ["trust-and-flow", "hub"],
   [F+"recall.feature:94"],
   "Counter and OPS-01 count only dropped, rejected, redacted, truncated, coalesced, timed-out or failed operations, shown on Health. A count of recalls is a stat read through `stat_list`, and no requirement names `recall_calls`.",
   "recall.feature:94: 'And the count of recalls `stat_list` reports increases by 1'.",
   [F+"recall.feature"])

fx("PIN-01 lists qualifying-pin sources too narrowly",
   ["pins-and-context#1"], ["pins-and-context"],
   [S+"05-functional-requirements.md:62", F+"pins.feature (PIN-01 Examples)"],
   "Qualifying pins also come from the owner's intent principal act (LANE-20, `parinn intent set|revise`) and from confirming a pin candidate (PIN-05, LANE-26); read as exhaustive, PIN-01 excludes both.",
   "PIN-01: '... only from a principal (a pin or intent it sets at a principal surface, a pin candidate it confirms, its configuration or its stamp), never a harness skill's call.' Optionally add the two rows to the PIN-01 Examples.",
   [S+"05-functional-requirements.md", F+"pins.feature"])

fx("PIN-05's 'in automation deployment mode' implies configuration may confirm pin candidates in interactive mode",
   ["pins-and-context#2"], ["pins-and-context"],
   [S+"05-functional-requirements.md:66", F+"pins.feature:72"],
   "Only the principal's own widening act confirms a pin candidate (Pin candidate, LANE-26). The scenario row tests only automation mode, where no candidate is detected, so it proves nothing.",
   "PIN-05: 'never by configuration'. pins.feature PIN-05: add row '| interactive | a setting that confirms pin candidates automatically | 1 | 1 |'.",
   [S+"05-functional-requirements.md", F+"pins.feature"])

fx("PEER-06 carries a room's qualifying pins without the stamps and edits that make them qualify",
   ["pins-and-context#3"], ["pins-and-context", "seats-and-keys"],
   [S+"05d-peer-requirements.md:22", F+"peer.feature:68"],
   "A run seat's pin qualifies only through a stamp, and a pin restores its newest version (Pin, LANE-32). Carrying only the pins' own events in the access token omits the stamps and edit events.",
   "PEER-06: '... MUST carry that room's qualifying pins (PIN-10) as the signed events they rest on: each pin's creating and edit events, keeping their provenance (`operator` for a device seat's pin, `assistant` for a run seat's), and every stamp that makes a version qualify ...'; mirror in peer.feature:68.",
   [S+"05d-peer-requirements.md", F+"peer.feature"], htr=True)

fx("CMP-05 folds the step budget into 'a limit'",
   ["pins-and-context#6"], ["pins-and-context", "record"],
   [S+"05-functional-requirements.md:124", F+"kernel.feature:56-69"],
   "The step budget is a named budget and the limits keep their own names; CMP-05 says 'Exceeding a limit', and the scenario yields 'the step budget limit was exceeded'.",
   "CMP-05: 'Exceeding the step budget or either limit MUST ...'. kernel.feature CMP-05: column `bound` with 'step budget', '10 s wall-clock limit', '512 MiB memory limit'; steps 'stating the <bound> was exceeded' and audit 'kernel <bound> exceeded'.",
   [S+"05-functional-requirements.md", F+"kernel.feature"])

fx("§8.4 groups the restore block limit with the output caps",
   ["pins-and-context#7"], ["pins-and-context"],
   [S+"08-data-and-storage.md:57"],
   "Budget names INJ-07's bound the restore block limit, distinct from the output caps of RCL-03 and CMP-06.",
   "§8.4: 'The pin budget (PIN-08), the restore block limit (INJ-07) and the output caps of RCL-03 and CMP-06 depend on model-token counts.'",
   [S+"08-data-and-storage.md"])

fx("§3 says 'the view' for the working view",
   ["pins-and-context#8"], ["pins-and-context"],
   [S+"03-rationale.md:13"],
   "Unqualified 'the view' reads as the room view, a different concept.",
   "§3: '**Keep the record separate from the working view.**'",
   [S+"03-rationale.md"])

fx("The bound ENG-01 scenario says 'pinned' and 'stamped' for the toolchain and release builds",
   ["pins-and-context#9"], ["pins-and-context", "acts-and-roles"],
   [F+"engineering.feature:11", F+"engineering.feature:13", "cmd/parinn/bdd_engineering_test.go:40"],
   "Pin and stamp are model words; ENG-01's own text avoids them ('fixed to an exact version'). This scenario is not @pending, so its step binding must change with it.",
   "engineering.feature:11: 'Scenario: the toolchain is fixed to an exact version and release builds are static, trimmed and carry VCS information'; :13 'Then \"go.mod\" fixes the Go toolchain with a toolchain directive'; the step regex at cmd/parinn/bdd_engineering_test.go:40 in the same commit.",
   [F+"engineering.feature", "cmd/parinn/bdd_engineering_test.go"])

fx("A provenance outline coins 'model-reproducible text' and 'harness-summarised text'",
   ["pins-and-context#10"], ["pins-and-context", "record"],
   [F+"provenance.feature:85"],
   "The phrases replace the model's model reply and `harness_text`, and 'harness-summarised' misdescribes examples that are no summaries (a system reminder, a restore block written back).",
   "provenance.feature:85: 'Scenario Outline: model replies and tool calls no principal stamped, and harness_text, are untrusted'.",
   [F+"provenance.feature"])

fx("T29 cites SEC-19 for the sandbox; it is OWN-22",
   ["places#2"], ["places"],
   [S+"06-security.md:70"],
   "Sandbox is defined under OWN-22; SEC-19 is the boundary register.",
   "T29: '... and a sandbox (OWN-22)'.",
   [S+"06-security.md"])

fx("OWN-11 says 'on the device' for the authenticator",
   ["places#3"], ["places"],
   [S+"05c-principal-and-peer-requirements.md:27"],
   "Device means a node or a paired phone. Read that way, an authenticator could rely on the host node's user verification, loosening which authenticators qualify against T21.",
   "OWN-11: 'certified to enforce user verification on the authenticator itself'.",
   [S+"05c-principal-and-peer-requirements.md"], htr=True)

fx("'Shared' and 'shares' are used for nodes holding a room",
   ["places#4", "places#8"], ["places"],
   [F+"security.feature:371", F+"peer.feature:145", M+"places.md:23", M+"places.md:41-42"],
   "Visibility's 'shared' means shared with the room's principals; the hub's verb for a node keeping a room is 'holds'. Two scenarios and places.md's node clone wording use 'shared'/'shares' for nodes.",
   "security.feature:371: 'And a room two enrolled peers hold, in which another principal has a seat'; peer.feature:145: 'And a room owned by \"alice\", held by her node and the nodes of \"bob\" and \"carol\"'; places.md: '... mints new seat keys and holds the carried-over personal room, as the original node does' and '... or carried over to a node clone, which holds it too'.",
   [F+"security.feature", F+"peer.feature", M+"places.md"])

fx("'Foreign history' uses 'foreign' outside rooms",
   ["places#5"], ["places"],
   [S+"02-context.md:20", S+"02-context.md:99", ".claude/agents/persona-oss-maintainer.md:40"],
   "Foreign is defined only for rooms. §2 and the OSS-maintainer persona, which the §2.5 gate keeps in step, say 'foreign history'.",
   "'a foreign room's history' in each place, kept in step per the §2.5 gate. The persona file is under CODEOWNERS (agent instructions).",
   [S+"02-context.md", ".claude/agents/persona-oss-maintainer.md"])

fx("LANE-02 says 'clone' beside 'node' for a git clone",
   ["places#10"], ["git-and-forge", "places"],
   [S+"05b-lane-requirements.md:15"],
   "Unqualified 'clone' next to 'node' reads as the model's node clone; the model's Repository entry says 'git clone'.",
   "LANE-02: 'every node holding a git clone', 'A node whose git clone ...' (twice).",
   [S+"05b-lane-requirements.md"])

fx("No requirement stops a personal-room seat leaving or being kicked",
   ["principals-and-agents#2"], ["principals-and-agents", "acts-and-roles"],
   [M+"principals-and-agents.md:70-71", S+"05b-lane-requirements.md:35", S+"09-interfaces.md:62"],
   "Member says a personal-room seat can neither leave nor be kicked, but LANE-23 makes every leave a room act and `room_leave` takes any room; before PRV-10, LANE-25 does not stop the owner's device seat kicking its run's personal-room seat. Either contradicts LANE-01's 'else to its personal-room seat, never refused'.",
   "LANE-23: 'A personal-room seat MUST NOT leave or be kicked; ParleyInn MUST refuse and audit either act.' Add a step to the LANE-23 scenario in lane.feature, and note the refusal on `room_leave` in 09-interfaces.md.",
   [S+"05b-lane-requirements.md", F+"lane.feature", S+"09-interfaces.md"])

fx("Principal says only an agent's own principal widens what reaches it, which cross-principal delegation contradicts",
   ["principals-and-agents#4"], ["principals-and-agents", "trust-and-flow"],
   [M+"principals-and-agents.md:18-19"],
   "Under OWN-26 and I2 a delegated task reaches another principal's agent under the delegating principal's delegation grant, a widening act of a different principal, beside the acceptance grant. 'Only' overstates I2.",
   "Model: 'Nothing widens what reaches an agent without its own principal's act (I2): another principal's delegation grant reaches it only beside its acceptance grant (OWN-26), and a room owner's notice allowance only lets through ...'.",
   [M+"principals-and-agents.md"])

fx("Timeline rails speak of 'the agent's principal' in a room of several principals",
   ["principals-and-agents#7"], ["principals-and-agents"],
   [S+"09b-lane-vocabulary.md:133-134"],
   "A room's Timeline holds agents of several principals, so 'the agent's principal' names no single principal; §9.7.6 is otherwise framed from the viewing principal.",
   "'Timeline rails: solid for the principal viewing, hollow for other principals, dotted for agents and the forge.'",
   [S+"09b-lane-vocabulary.md"])

fx("§2 says runs 'pause and resume' for a harness resume",
   ["principals-and-agents#8"], ["principals-and-agents"],
   [S+"02-context.md:37-38"],
   "Pause and resume are run controls (principal acts). The durable-volume context means a harness resume, which continues a run only when it keeps its harness session (Run, REC-02).",
   "'... live on durable volumes so a harness session can be resumed (a harness resume, REC-02).'",
   [S+"02-context.md"])

fx("A record scenario expects two homes to share a writer id and chain head",
   ["record#1"], ["record", "seats-and-keys"],
   [F+"record.feature:145"],
   "A writer id derives from the seat's first key, generated on and bound to one node (SEC-10, REC-24), and the chained header holds a commitment under a random per-event commitment key (REC-17). Two ingests never share either, so the step cannot pass.",
   "record.feature:145: 'And a second fresh home that ingests \"main-run\" assigns its lines the same seq order'.",
   [F+"record.feature"])

fx("A record scenario excludes the seat ingest starts from the core's sealing",
   ["record#2"], ["record"],
   [F+"record.feature:226"],
   "A witnessed run's run seats are sealed by its MCP server except the seat ingest starts, which the core seals; line 222 of the same scenario says so, line 226 contradicts it.",
   "record.feature:226: '... every writer but the witnessed run's run seats' other than those \"parinn ingest\" started, and the paired phone's, ...'.",
   [F+"record.feature"])

fx("An ADM-15 row refuses a single event where peers exchange segments, and omits the structural event",
   ["record#3"], ["record"],
   [F+"administration.feature:205"],
   "Peers exchange sealed ranges, not single events (PEER-03), and ADM-15 records each refused segment as REC-21 says; line 206 includes that, line 205 does not.",
   "administration.feature:205: '| received writer | a peer offers another segment of that writer | the segment is refused, and a structural event and an audit entry record the refusal |'.",
   [F+"administration.feature"])

fx("Integrity status is defined per room or writer, but VIEW-10, REC-18 and RCL-09 give one per event",
   ["record#4", "record#5"], ["record"],
   [S+"09-interfaces.md:122-123", M+"record.md:152", S+"09b-lane-vocabulary.md:98"],
   "The envelope's `integrity` is 'the integrity status of the event's writer', and the model and §9.7.5 define integrity status for rooms and writers. VIEW-10 marks every later event of a broken writer `unverified`, REC-18 makes events past the newest seal `unsigned`, and the RCL-09 scenario shows different values per event of one writer.",
   "09-interfaces.md:122-123: '`integrity` is the event's integrity status, one of §9.7.5's values'; record.md:152: 'What a room, writer or event shows about its chain and seals'; 09b:98: 'The integrity status of a room, a writer or an event'.",
   [S+"09-interfaces.md", M+"record.md", S+"09b-lane-vocabulary.md"], htr=True)

fx("PRV-01 classes every key rotation `structural`, but a device-key rotation is an `operator` principal act",
   ["record#6"], ["record", "seats-and-keys", "acts-and-roles"],
   [S+"05-functional-requirements.md:46", M+"record.md:75"],
   "Rotating a device key is a widening principal act (OWN-11, `parinn device-key rotate`), recorded as `operator` (OWN-02). PRV-01 and record.md:75 make 'key rotations' `structural`, a second class for the same event; the class sits in the chained header (REC-10) and decides trust on the principal's other nodes.",
   "PRV-01 and record.md:75: 'key rotations' -> 'seat-key rotations' (a seat key's rotation goes to its own writer, SEC-27; a device-key rotation stays an `operator` principal act).",
   [S+"05-functional-requirements.md", M+"record.md"], htr=True)

fx("PEER-12, §6.3 row 27 and a peer scenario say 'range' for a sealed range",
   ["record#10"], ["record"],
   [S+"05d-peer-requirements.md:28", S+"06b-boundary-register.md:58", F+"peer.feature:136-137"],
   "'Range' on its own means an address range. A blind peer holds and verifies sealed ranges, and the same row uses 'address ranges' for the metadata it may learn.",
   "'sealed range(s)' in PEER-12 (three times), 06b:58, peer.feature:136 and :137.",
   [S+"05d-peer-requirements.md", S+"06b-boundary-register.md", F+"peer.feature"])

fx("REC-17 calls the commitment key only 'a random per-event secret'",
   ["record#11"], ["record"],
   [S+"05-functional-requirements.md:32"],
   "The model, REC-23 and ADM-07 name `k_e` the commitment key; REC-17, the requirement the model cites for it, does not, a second name for one concept.",
   "REC-17: '... under its commitment key `k_e`, a random per-event secret of at least 256 bits† ...'.",
   [S+"05-functional-requirements.md"])

fx("The model's Payload promises more about the payload's name than REC-09",
   ["record#12"], ["record"],
   [M+"record.md:44-45"],
   "REC-09 and its scenario promise only a name 'from which no one off this node can confirm a guess at the content'; the model drops 'off this node'.",
   "record.md:44-45: 'under a name from which no one off this node can confirm a guess at its content (REC-09)'.",
   [M+"record.md"])

fx("The model's Flag covers every event; PRV-07 flags only untrusted ones",
   ["record#13"], ["record"],
   [M+"record.md:138-139"],
   "PRV-07: 'ParleyInn SHOULD flag ... untrusted events that contain instruction-like content'.",
   "record.md:138: 'A mark ParleyInn sets on an untrusted event whose text matches an injection pattern (PRV-07)'.",
   [M+"record.md"])

fx("record.md calls one event 'a witness check's record'",
   ["record#14"], ["record", "git-and-forge"],
   [M+"record.md:76"],
   "Record means the set of writer logs a node holds; here it names one event, inside the file that defines it.",
   "record.md:76: '... tombstones and the event recording a witness check (its command by commitment, its exit status and the tree hash, OWN-18) are `structural`'.",
   [M+"record.md"])

fx("LANE-23 derives the seat id from 'that seat key', which changes after a rotation",
   ["seats-and-keys#1"], ["seats-and-keys"],
   [S+"05b-lane-requirements.md:35"],
   "Seat id derives from the room id and the seat's first key, and a rotation keeps the seat's id (SEC-27, LANE-23's own 'MUST keep its seat id and writer'). 'That seat key' is the key the seat joins with, which after a rotation is the current one; it also leaves seats with no join (create room, personal room) underived.",
   "LANE-23: 'The seat id MUST be derived from the room's id and the seat's first seat key.'",
   [S+"05b-lane-requirements.md"], htr=True)

fx("LANE-23's seat-certificate chain to a principal key is unhedged before PRV-10",
   ["seats-and-keys#2"], ["seats-and-keys"],
   [S+"05b-lane-requirements.md:35"],
   "Before PRV-10 ships no principal key certifies a device key (Device key), yet LANE-01 (P0, M1) already has a device seat join a room; such a join cannot meet LANE-23's 'certified by a seat certificate that chains to its principal key'.",
   "LANE-23: 'certified by its seat certificate (PRV-11), which, once PRV-10 ships, chains to its principal key'.",
   [S+"05b-lane-requirements.md"])

fx("Room summaries are 'signed with that device seat' rather than its seat key",
   ["seats-and-keys#3"], ["seats-and-keys"],
   [S+"06-security.md:122", S+"09a-command-line-interface.md:57", F+"security.feature:402", F+"lane.feature:517"],
   "A seat signs nothing; a room summary is a room act signed with the seat key (LANE-31). 'Signed with that device seat' can be read as signed with the device key, which would make it a principal act. 01b-scope.md:30 has it right.",
   "In all four places: 'signed with that device seat's seat key' (or '... its device seat's seat key').",
   [S+"06-security.md", S+"09a-command-line-interface.md", F+"security.feature", F+"lane.feature"])

fx("§6.3 rows 19 and 27 omit the token key segments may be encrypted to",
   ["seats-and-keys#4"], ["seats-and-keys"],
   [S+"06b-boundary-register.md:50", S+"06b-boundary-register.md:58"],
   "PEER-08 and PEER-12 allow encryption to a token-key-only node's token key, and row 15 says so; rows 19 and 27 name only device keys, understating who can read the segments.",
   "06b rows 19 and 27: add ', or a token-key-only node's token key' after 'device keys'.",
   [S+"06b-boundary-register.md"])

fx("A recall scenario labels a seat \"bob\", a principal's name elsewhere",
   ["seats-and-keys#5"], ["seats-and-keys"],
   [F+"recall.feature:153", F+"recall.feature:156"],
   "A seat is shown by its id, never a claimed name (LANE-23); every other scenario uses \"bob\" for a principal and labels seats p-1, s-1.",
   "RCL-11 scenario: 'a post written by a device seat of \"bob\"' and 'to \"owner-a\" and to \"bob\"' (or label the seat \"p-1\").",
   [F+"recall.feature"])

fx("`room_bar` takes a bare `key`",
   ["seats-and-keys#6"], ["seats-and-keys", "hub"],
   [S+"09-interfaces.md:70"],
   "An option that selects a concept is named for it, multi-word concepts in snake case for MCP tools; every other room-tool parameter is named for its concept, and a bare `key` does not say which key.",
   "§9.2 `room_bar`: rename the parameter to `principal_key`.",
   [S+"09-interfaces.md"])

fx("seats-and-keys.md has the access token carry the invite link, the wrong way round",
   ["seats-and-keys#9"], ["seats-and-keys", "git-and-forge"],
   [M+"seats-and-keys.md:77-79"],
   "git-and-forge's Qualified links ('an invite link ... carries an access token') and LANE-18 ('an invite link carrying a one-time access token') reverse it.",
   "seats-and-keys.md: '... certify an ephemeral node's seats, or travel in an invite link (PEER-06, PRV-10, LANE-18)'.",
   [M+"seats-and-keys.md"])

fx("Token key's limits cite PRV-10; PEER-06 states them",
   ["seats-and-keys#10"], ["seats-and-keys"],
   [M+"seats-and-keys.md:64-67"],
   "PRV-10 says nothing about a token key's limits; PEER-06 states them.",
   "seats-and-keys.md: cite '(PEER-06, PRV-10)'.",
   [M+"seats-and-keys.md"])

fx("The hub's 'a new one for each seat key minted anew' can be read as covering rotations",
   ["seats-and-keys#12"], ["hub", "seats-and-keys"],
   [M+"index.md:58-60"],
   "A rotation keeps the seat's id and writer (SEC-27); only a key minted on a node identity change, a node clone or a backup restore starts a new seat.",
   "Hub: 'a new one for each seat key minted on a node identity change, a node clone or a backup restore (Seat key)'.",
   [M+"index.md"])

fx("OWN-28 uses 'correction' both for the act and as an umbrella over revision and retry",
   ["trust-and-flow#1"], ["trust-and-flow"],
   [S+"05c-principal-and-peer-requirements.md:44"],
   "Correction is principal-typed text in a fixed template; OWN-28 also calls intent revision and retry corrections.",
   "OWN-28: 'each a principal act naming the verdict it answers: send a correction ...'.",
   [S+"05c-principal-and-peer-requirements.md"])

fx("OQ-14 says 'delegation' for one principal instructing another's agent",
   ["trust-and-flow#2"], ["trust-and-flow"],
   [S+"13-open-questions-and-risks.md:25"],
   "Delegation is one agent handing another a delegated task; the same row then uses the right sense for OWN-26.",
   "OQ-14: 'Default: only the agent's own principal instructs it (OWN-01); requires its own I2 review.'",
   [S+"13-open-questions-and-risks.md"])

fx("§11.1 says 'recall' for the classifier statistic",
   ["trust-and-flow#4"], ["trust-and-flow"],
   [S+"11-verification-and-acceptance.md:26"],
   "Recall is an agent's call to a recall tool.",
   "§11.1: 'Precision ≥ 0.9†; share of malicious items flagged reported'.",
   [S+"11-verification-and-acceptance.md"])

fx("ENG-29 omits OWN-26 from the requirements marked for I2 review",
   ["trust-and-flow#5"], ["trust-and-flow"],
   [S+"10-engineering-quality.md:59", F+"principal-acts.feature:309-319"],
   "OWN-26 is marked for I2 review under ENG-29, but ENG-29 lists only PRV-10 and PRV-11, and the OWN-26 scenario lacks the review step the PRV-10 and PRV-11 scenarios carry.",
   "ENG-29: '(PRV-10, PRV-11, OWN-26)'; OWN-26 scenario: add 'And a recorded I2 security review of this requirement exists before it ships'.",
   [S+"10-engineering-quality.md", F+"principal-acts.feature"])

fx("Counter says failures show on Health only until acknowledged; ADM-11 keeps them shown",
   ["trust-and-flow#9"], ["trust-and-flow"],
   [M+"trust-and-flow.md:140-141", S+"05a-administration-requirements.md:24", S+"09a-command-line-interface.md:16"],
   "ADM-11 and `parinn status` keep non-zero failure counters shown; OPS-03 uses acknowledgement only as `parinn doctor`'s baseline.",
   "Model: 'shown on Health; `parinn doctor` fails while one has risen since the node's principal last acknowledged them (`parinn counter ack`, I6, OPS-03)'.",
   [M+"trust-and-flow.md"])

fx("Continuation cursor is promised for every capped recall tool, and RCL-03 lacks 'without gap or overlap'",
   ["trust-and-flow#10"], ["trust-and-flow", "record"],
   [M+"trust-and-flow.md:52-54", S+"05-functional-requirements.md:80"],
   "`kernel_exec`, a recall tool, returns a truncation marker at its cap (CMP-06), and `landmark_list` has a cap but no cursor. RCL-03 does not state 'without gap or overlap'; only recall.feature:48 does.",
   "Model: '`event_expand` or `event_get`, stopping at its per-call cap, returns a continuation cursor ...'; RCL-03: 'with a continuation cursor that continues without gap or overlap'.",
   [M+"trust-and-flow.md", S+"05-functional-requirements.md"])

fx("Focus set puts rooms first in Needs you; §9.7.4 orders by class first",
   ["trust-and-flow#11"], ["trust-and-flow", "components-and-surfaces"],
   [M+"trust-and-flow.md:159-160"],
   "§9.7.4 (09b:88-90) puts focus-set rooms first only inside Q2 and Q3.",
   "Model: 'The rooms a principal marks to come first inside Needs you's Q2 and Q3 (§9.7.4)'.",
   [M+"trust-and-flow.md"])

fx("Quota omits that repository configuration may lower it",
   ["trust-and-flow#13"], ["trust-and-flow"],
   [M+"trust-and-flow.md:108-110"],
   "§9.6 lets repository configuration lower `quota.*`.",
   "Model: '... that the node's principal or managed policy sets, and repository configuration may only lower (ADM-15, §9.6)'.",
   [M+"trust-and-flow.md"])

fx("Petname omits names a peer sent",
   ["trust-and-flow#14"], ["trust-and-flow"],
   [M+"trust-and-flow.md:176-178"],
   "VIEW-07: 'never a name another principal or a peer sent'.",
   "Model: 'never one another principal or a peer sent (VIEW-07)'.",
   [M+"trust-and-flow.md"])

fx("trust-and-flow.md's summary claims the Needs you queue, which components-and-surfaces.md defines",
   ["trust-and-flow#15"], ["trust-and-flow", "hub"],
   [M+"trust-and-flow.md:5", M+"index.md (catalog row)"],
   "The summary, and the hub catalog row generated from it, point here for the Needs you queue; this file defines only queue classes and the focus set.",
   "Front matter summary: 'Needs you's queue classes and the focus set'; then `mdsmith fix` regenerates the hub catalog.",
   [M+"trust-and-flow.md", M+"index.md"])

fx("PRV-02's trust-level inputs omit whether the event's room is foreign",
   ["whole#2"], ["trust-and-flow", "places"],
   [S+"05-functional-requirements.md:47"],
   "Trust level lists 'whether the event's room is foreign to that principal' among its inputs; PRV-02 omits it, though PRV-02's own foreign-room rule depends on it.",
   "PRV-02: insert 'whether the event's room is foreign to that principal,' before 'the node's key set'.",
   [S+"05-functional-requirements.md"])

fx("§9.5 'publish ... to a host' reads as an outbound send",
   ["whole#5"], ["record", "components-and-surfaces"],
   [S+"09a-command-line-interface.md:37"],
   "To publish is to serve a room's bundle read-only through the publish component's inbound listener (§6.3 row 18); outbound exchange with hosts is the bridge component's.",
   "§9.5 row: 'publish a room's bundle, served read-only by the publish component's listener (B3) at a host the node's principal names, ...'.",
   [S+"09a-command-line-interface.md"])

fx("§9.7.4 says a pin version was edited or unpinned",
   ["whole#6"], ["pins-and-context"],
   [S+"09b-lane-vocabulary.md:82-83"],
   "A pin version is one immutable text; edits, unpins and list removals act on the pin (Stamp, LANE-32).",
   "§9.7.4: 'a pin whose version you stamped was edited, unpinned or taken off the pin list (LANE-32)'.",
   [S+"09b-lane-vocabulary.md"])

fx("VIEW-21 gives agents the Idle run status",
   ["whole#7"], ["trust-and-flow", "principals-and-agents"],
   [S+"05b-room-view-requirements.md", F+"lane-view.feature:58", F+"lane-view.feature:218"],
   "Run status, Idle included, belongs to a run; agents carry no status. The scenario repeats it ('When its agents go idle').",
   "VIEW-21: 'A room whose runs all show Idle while ...'; scenario: 'When its runs all show Idle and ...'.",
   [S+"05b-room-view-requirements.md", F+"lane-view.feature"])

fx("OWN-27 says the room records no verdict; rooms record nothing",
   ["whole#8"], ["places", "git-and-forge"],
   [S+"05c-principal-and-peer-requirements.md:43"],
   "A room is a place, not an actor (nodes record), and Verdict says ParleyInn never derives one (VIEW-22).",
   "OWN-27: 'ParleyInn records no verdict of its own (VIEW-22).'",
   [S+"05c-principal-and-peer-requirements.md"])

fx("ENG-02's example crate `adapter-claudecode` drops the model's 'harness adapter'",
   ["whole#9"], ["components-and-surfaces", "hub"],
   [S+"10-engineering-quality.md:17"],
   "Crate names use the model's words, and 'adapter' alone is not one.",
   "ENG-02: rename the example crate `harness_adapter_claude_code` (matching `restore_block`'s form).",
   [S+"10-engineering-quality.md"])

fx("ADR-06 in §4.5 says `parinn install` edits 'settings', which reads as ParleyInn's configuration",
   ["whole#10"], ["principals-and-agents", "record"],
   [S+"04-reference-architecture.md:157"],
   "Configuration is the principal's settings (§9.6); the harness's settings are part of harness configuration.",
   "§4.5 ADR-06: '`parinn install` edits the harness's settings only as a fallback.'",
   [S+"04-reference-architecture.md"])
