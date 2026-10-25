# Part 1: stakeholder and decide clusters.
S = "docs/srs/"
M = "docs/domain-model/"
F = "features/"

def c(title, rids, owners, locations, problem, fix, cls, conflict="", options=None, htr=False, touches=None):
    return dict(title=title, rids=rids, owners=owners, locations=locations, problem=problem,
                proposed_fix=fix, cls=cls, conflict=conflict, options=options or [],
                hard_to_revert=htr, touches=touches or [])

CLUSTERS = []
A = CLUSTERS.append

# ---------------- stakeholder ----------------
A(c("I2 lets compaction guidance carry structural fields and ids; the model and PIN-07 allow only fixed text",
    ["harness-facts#8"], ["harness-facts", "trust-and-flow"],
    [S+"invariants.md:37-39", M+"harness-facts.md:44-46", S+"05-functional-requirements.md:68", S+"09-interfaces.md:20", S+"04-reference-architecture.md:128-129", M+"index.md:36"],
    "I2 groups compaction guidance (PIN-07) with opt-in notices and the OWN-04/OWN-07 templates, all 'built only from fixed text ParleyInn ships, trusted structural fields and ids'. The model's Compaction guidance, PIN-07, §9.1, §4.4 and the fifteenth-round decision all say compaction guidance is fixed text ParleyInn ships and nothing else. The invariant and the model therefore do not read the same.",
    "Narrow I2 by ADR and security review (ENG-29): '... and compaction guidance (PIN-07), built only from fixed text ParleyInn ships; and opt-in notices (INJ-10) and the fixed templates of OWN-04 and OWN-07, each built only from fixed text ParleyInn ships, trusted structural fields and ids.' Then run `mdsmith fix .` so CLAUDE.md's include follows.",
    "stakeholder", "",
    ["Narrow I2 as proposed (ADR, security review, mdsmith fix for CLAUDE.md); model and PIN-07 stay.",
     "Keep I2 as the ceiling and record in the model's Compaction guidance entry that PIN-07 deliberately allows less than I2 does; no invariant change."],
    True, [S+"invariants.md", "CLAUDE.md", "docs/adr/ (new ADR)"]))

A(c("I2's 'its principal recorded' binds a cross-principal delegation grant to the receiving agent's principal",
    ["principals-and-agents#3"], ["principals-and-agents", "trust-and-flow"],
    [S+"invariants.md:42-44", "CLAUDE.md (I2 include)", S+"05c-principal-and-peer-requirements.md:39", S+"05c-principal-and-peer-requirements.md:42", M+"trust-and-flow.md:89-96"],
    "I2 says 'Under a delegation grant its principal recorded (OWN-23), and, for an agent of another principal, an acceptance grant that agent's principal recorded (OWN-26)'. I2 never names the delegating agent, so 'its principal' binds to the agent written to. For cross-principal delegation that has the receiving principal record the delegation grant, contradicting OWN-23, OWN-26 and the model's Delegation grant (the delegating principal's widening act).",
    "Reword I2 by ADR and security review (ENG-29): 'Under a delegation grant the delegating agent's principal recorded (OWN-23), and, when the receiving agent has another principal, an acceptance grant that principal recorded (OWN-26): a delegated task inside the fixed template of OWN-24.' Then run `mdsmith fix .` for CLAUDE.md.",
    "stakeholder", "", ["Reword I2 as proposed, under one ADR together with C01 if both go ahead."],
    False, [S+"invariants.md", "CLAUDE.md", "docs/adr/ (new ADR)"]))

# ---------------- decide ----------------
A(c("A joined run seat defaults to viewer: two scenarios route events to it, and the owner's own run gets no work in its own room",
    ["acts-and-roles#1", "acts-and-roles#2", "acts-and-roles#7"], ["acts-and-roles", "principals-and-agents", "places"],
    [F+"lane.feature:11", F+"lane.feature:15", F+"lane.feature:17", F+"landmarks.feature:13", M+"acts-and-roles.md:113-115", S+"05b-lane-requirements.md:28", S+"05b-lane-requirements.md:14"],
    "Role and LANE-16 make a seat with no role assignment or appointment a viewer (only a run's personal-room seat, or its seat in a room it created, is a contributor), and 'No role MAY follow from joining'. The LANE-01 and LMK-01 scenarios have a run join a room and then route its events to that seat, which a viewer with no work cannot receive; lane.feature:17 also implies the default is not viewer. The same rule leaves the owner's own run with no work in the owner's own room unless the owner assigns a role after every join, since the seat id exists only after the join.",
    "Rule on the default role of the owner's own run seats, then fix both scenarios to match: state who owns R and L1 and, unless the default covers it, add an invite giving the seat the contributor role; lane.feature:17 reads 'while a later role assignment gives the run's seat in \"R\" the viewer role'.",
    "decide", "",
    ["A seat of the owner's principal's run in a room that principal owns is a contributor (Role entry, LANE-16's 'Except ...' sentence); scenarios state that R and L1 are owned by \"alice\"'s principal; lane.feature:17 says 'a later role assignment'.",
     "Keep the viewer default; let the owner invite its own principal key (LANE-18, model Role) and have lane.feature:11 and landmarks.feature:13 add 'whose owner invited \"alice\"'s principal key as contributor' / 'under an invite that gives its seat the contributor role'.",
     "Keep the viewer default and no new act; scenarios add an explicit role assignment by the owner after the join."],
    True, [F+"lane.feature", F+"landmarks.feature", M+"acts-and-roles.md", S+"05b-lane-requirements.md"]))

A(c("LANE-19's 'concurrently' uses the model's causal word for overlap in time",
    ["acts-and-roles#3"], ["acts-and-roles", "principals-and-agents"],
    [S+"05b-lane-requirements.md:31", F+"lane.feature (LANE-19 scenario)"],
    "The model defines Concurrent only for acts and events: neither causally after the other. Two subagents' edit events on one node are causally ordered through the writer heads each records (REC-10), so 'edit the same file concurrently' never fires as written. The requirement means overlap in time, and the window it should cover is a behaviour choice.",
    "Reword LANE-19's trigger in time terms and mirror it in its scenario.",
    "decide", "",
    ["'When two subagents that share the worktree both edit the same file while both run, ParleyInn MUST raise a Needs you item naming both.'",
     "Bound the window by worktree checkpoints: '... both edit the same file between two worktree checkpoints ...'."],
    False, [S+"05b-lane-requirements.md", F+"lane.feature"]))

A(c("`parinn ui --phone` takes a widening principal act outside the CLI and TUI, and 'CLI' has two meanings",
    ["components-and-surfaces#1"], ["components-and-surfaces", "trust-and-flow", "hub"],
    [S+"05c-principal-and-peer-requirements.md:32", S+"05c-principal-and-peer-requirements.md:18", M+"components-and-surfaces.md:12-13", M+"components-and-surfaces.md:31-33", M+"trust-and-flow.md:40-41", S+"09a-command-line-interface.md:80", S+"04-reference-architecture.md:65-67", S+"06b-boundary-register.md:39", F+"principal-acts.feature:204", M+"index.md:108-111"],
    "OWN-16 makes issuing a phone-scoped room-view secret (`parinn ui --phone`) a widening principal act at a terminal. The model's Core says the CLI is `parinn` but for `parinn ui`, the Room-view component records only the browser room view's principal acts, and OWN-02 and Principal surface accept terminal acts only from the CLI or TUI, refusing any other. The hub's Names section meanwhile counts `ui` and `launch` among CLI commands, giving 'CLI' two meanings.",
    "Choose where the act is taken; either way qualify the hub's Names list so `ui` and `launch` are commands of the room-view component and the launcher, not the core's CLI.",
    "decide", "The reviewer listed (a) first; (b) is put first here because it keeps OWN-02's closed set of principal surfaces inside the core (pick safety).",
    ["(b) Move the issue into a core CLI verb named per the hub (`parinn <concept> <verb>`) that records the act; `parinn ui --phone` only reads it from the record. Edit OWN-16, §9.5 (09a:80), §4.2 (04:65-67) and principal-acts.feature:204.",
     "(a) Add to the model's Room-view component 'and, at the terminal that starts it, the widening act issuing a phone-scoped room-view secret (OWN-16)'; add that entry point to OWN-02's surface list and to trust-and-flow.md's Principal surface; add it to §6.3 row 8's Reach (signing under OQ-40)."],
    True, [S+"05c-principal-and-peer-requirements.md", S+"09a-command-line-interface.md", S+"04-reference-architecture.md", S+"06b-boundary-register.md", F+"principal-acts.feature", M+"components-and-surfaces.md", M+"trust-and-flow.md", M+"index.md"]))

A(c("The peer and publish components record refusal events, but are neither recorders in the model nor covered by OQ-40",
    ["components-and-surfaces#2", "record#7"], ["record", "components-and-surfaces"],
    [S+"06-security.md:115", S+"13-open-questions-and-risks.md:47", S+"06b-boundary-register.md:25-26", M+"record.md:79-91", S+"05a-administration-requirements.md (ADM-15)", S+"05-functional-requirements.md (PRV-09)"],
    "SEC-25 has the peer component (B2) refuse a segment and record it 'as REC-21 says', a structural event in this node's device seat writer; ADM-15 and PRV-09 imply the same for the git carrier in the publish component. The model's recorder list (the closed set the trust policy reads) names neither component, REC-19 and SEC-10 leave sealing to the core, and OQ-40 and the §6.3 note ask who signs and seals only for the room-view component, the launcher and the bridge component.",
    "Decide who writes the REC-21 event, then align SEC-25, ADM-15, PRV-09, record.md's recorder list, OQ-40 and the §6.3 note.",
    "decide", "Both reviewers offer the same two fixes in opposite order: components-and-surfaces#2 leads with widening OQ-40, record#7 with routing the refusal to the core.",
    ["SEC-25 (and ADM-15, PRV-09 for the git carrier): '... MUST refuse and audit a segment failing either and pass the refusal to the core, which records it as REC-21 says'. The recorder list and OQ-40 stay as they are; the core stays the only sealer.",
     "Add the peer and publish components to record.md's recorder and witnessed lists; widen OQ-40 to 'the room-view component, the launcher, and the peer, publish and bridge components'; name §6.3 rows 12 and 19 in the note at 06b:25-26."],
    True, [S+"06-security.md", S+"05a-administration-requirements.md", S+"05-functional-requirements.md", S+"13-open-questions-and-risks.md", S+"06b-boundary-register.md", M+"record.md"]))

A(c("CON-03 admits a webview client the model's Room view and §6.3 do not list",
    ["components-and-surfaces#3"], ["components-and-surfaces"],
    [S+"02-context.md:77", M+"components-and-surfaces.md:57"],
    "CON-03 says the browser room view client's TypeScript runs 'in a webview or a browser'. The model's Room view lists the browser served through the room-view component plus the reduced clients; a webview is in neither the model nor §6.3, and nothing says whether it reaches the record through the room-view component on loopback under SEC-20. This runs ahead of the still-proposed ADR-2610042341.",
    "Either hold CON-03 back until the ADR's SRS changes land, or define when a webview counts as the browser.",
    "decide", "",
    ["Drop 'a webview or' from CON-03 until ADR-2610042341 is accepted and its SRS changes land.",
     "Add to components-and-surfaces.md Room view: 'a webview counts as the browser only when it loads the browser room view from the room-view component on loopback under SEC-20'."],
    False, [S+"02-context.md", M+"components-and-surfaces.md"]))

A(c("'Branch head' is undefined, and two scenarios stale a ready mark and a verdict on an uncommitted edit",
    ["git-and-forge#1"], ["git-and-forge"],
    [F+"principal-acts.feature:256-258", F+"principal-acts.feature:325-328", M+"git-and-forge.md:52", M+"git-and-forge.md:74-75", M+"git-and-forge.md:82", S+"05c-principal-and-peer-requirements.md:37", S+"05c-principal-and-peer-requirements.md:43"],
    "git-and-forge.md uses a branch's head without defining it. OWN-21 clears a ready mark only on 'a later change to any of those branch heads' and OWN-27 stales a verdict when 'any branch head, the intent version or the evidence it was bound to changes'. In git's sense a head is the tip commit, which an uncommitted edit does not change, yet the OWN-21 and OWN-27 scenarios clear and stale on an uncommitted edit.",
    "Define head in the Branch entry and decide what OWN-21 and OWN-27 bind.",
    "decide", "",
    ["Keep git's meaning ('its **head** is its tip commit; an uncommitted edit changes no head') and widen OWN-21 and OWN-27 to also clear or stale on a change to the tree of a worktree on a branch the room names (latest worktree checkpoint plus recorded edits, LANE-05); scenarios stay.",
     "Keep git's meaning and OWN-21/OWN-27 as written; both scenarios make the agent commit the edit ('the agent then commits an edit')."],
    False, [M+"git-and-forge.md", S+"05c-principal-and-peer-requirements.md", F+"principal-acts.feature"]))

A(c("The model says ParleyInn never lands, while OQ-37 leaves that open",
    ["git-and-forge#4"], ["git-and-forge"],
    [M+"git-and-forge.md:22", M+"git-and-forge.md:51", S+"13-open-questions-and-risks.md:44"],
    "git-and-forge.md states 'It approves and lands; ParleyInn does neither' and 'ParleyInn never lands anything' as permanent. OQ-37 ('May ParleyInn run, push or merge code itself? Until this closes, ADM-13 applies') leaves it open, so the model runs ahead of the SRS.",
    "Hedge the model to OQ-37, or close OQ-37's merge part.",
    "decide", "",
    ["Model, both places: 'ParleyInn lands nothing while OQ-37 is open (ADM-13)'.",
     "Close OQ-37's merge part (ParleyInn never lands) and keep the model as is."],
    False, [M+"git-and-forge.md", S+"13-open-questions-and-risks.md"]))

A(c("The Hook entry names hook input but not hook output or a hook's firing ('hook event'), which normative text relies on",
    ["harness-facts#1", "harness-facts#2"], ["harness-facts", "record", "git-and-forge"],
    [M+"harness-facts.md:28-30", S+"05c-principal-and-peer-requirements.md:19-20", S+"05b-room-view-requirements.md:54", S+"06b-boundary-register.md:37", S+"04-reference-architecture.md:76", F+"restore.feature:48-49", F+"principal-acts.feature:45-51", F+"lane-view.feature:186", F+"engineering.feature:40", F+"lane.feature:368", F+"assumptions.feature:29", S+"05b-lane-requirements.md:15", F+"lane.feature:37", F+"lane.feature:39", F+"assumptions.feature:156", S+"02-context.md:65", M+"git-and-forge.md:12"],
    "Hook defines hook input but nothing names what the handlers return, yet 'hook output' carries rules in OWN-03, OWN-04, VIEW-17, §6.3 row 6, §4.2 and six scenarios. 'Hook event' (LANE-02, ASM-17, three scenarios, git-and-forge.md:12) names one firing of a hook, while the hub gives 'event' one meaning, an entry in a writer. Whether harness-native words like these are outside terms or need model entries is unsettled.",
    "Name hook output in the Hook entry and settle 'hook event'.",
    "decide", "",
    ["Hook entry: '... the core's hook handlers answer it with their **hook output**, at most one JSON object (§9.1)'; reword 'hook event' everywhere: 'at the first hook', 'When the first hook for the clone fires', 'the time each hook fired', ASM-17 'within 1 s of the hook firing', git-and-forge.md:12 likewise.",
     "Hook entry names both hook output and hook event ('one firing of a hook, the harness's own term'); no SRS change.",
     "Treat both as harness-native outside terms under the hub's outside-thing rule and add them to the hub's examples."],
    False, [M+"harness-facts.md", S+"05b-lane-requirements.md", S+"02-context.md", F+"lane.feature", F+"assumptions.feature", M+"git-and-forge.md"]))

A(c("PIN-07 gates compaction guidance on ASM-01/02, which do not cover PreCompact output",
    ["harness-facts#7"], ["harness-facts", "pins-and-context"],
    [S+"05-functional-requirements.md:68", S+"02-context.md:49-50", F+"assumptions.feature:27-37", S+"13-open-questions-and-risks.md (OQ-02)"],
    "PIN-07 says 'If ASM-01/02 permit, the PreCompact hook handler SHOULD return compaction guidance'. Neither ASM-01 nor ASM-02, nor the ASM-02 scenario (no PreCompact row), says PreCompact output reaches the harness, and ASM-02's Affects lists only INJ-*. The open question is OQ-02 (spike S1).",
    "Point PIN-07 at the right gate.",
    "decide", "",
    ["PIN-07: 'If spike S1 finds that PreCompact output reaches the harness's compaction (OQ-02), the PreCompact hook handler SHOULD return compaction guidance ...'.",
     "Add PreCompact to ASM-02 with PIN-07 in its Affects column and a row in the ASM-02 scenario of assumptions.feature."],
    False, [S+"05-functional-requirements.md", S+"02-context.md", F+"assumptions.feature"]))

A(c("LANE-28 traces I7, but git's commit hook is not harness configuration",
    ["harness-facts#9", "principals-and-agents#5"], ["harness-facts", "principals-and-agents"],
    [S+"05b-lane-requirements.md:40", F+"lane.feature:441", S+"appendix-b-invariant-coverage.md:23"],
    "LANE-28 traces I7 (Traces column, @I7 tag, Appendix B). I7 covers only harness configuration (the harness's settings, hooks and MCP registrations); the repository's commit hook is git's hook, not a harness hook, and ADM-02 lists them separately. So no invariant guarantees what LANE-28 relies on.",
    "Drop the trace, or widen I7.",
    "decide", "",
    ["Drop I7 from LANE-28's Traces and the scenario's @I7 tag (lane.feature:441); the Appendix B test then needs the row regenerated. LANE-28 still traces I2, I4, I10.",
     "Widen I7 and the model's harness configuration to name the repository's commit hook (ADR and security review; stakeholder)."],
    False, [S+"05b-lane-requirements.md", F+"lane.feature", S+"appendix-b-invariant-coverage.md"]))

A(c("Foreign room versus the run and room recall scopes: rooms that turned foreign and cross-room posts' sending rooms",
    ["hub#2", "places#6"], ["places", "trust-and-flow", "hub"],
    [S+"05-functional-requirements.md:82", S+"05-functional-requirements.md:87", M+"places.md:48-56", M+"index.md:96-98", S+"05b-lane-requirements.md:41", S+"05d-peer-requirements.md:18", S+"13-open-questions-and-risks.md (OQ-41)"],
    "RCL-05's default run scope reaches 'every writer of its seats', including a run seat's writer in a room that has since turned foreign, while RCL-10 reaches a foreign room only by naming it. RCL-05's room scope covers 'the cross-room posts it shows', but a node that holds the sending writer only to show the post meets the Foreign room definition, and RCL-05 bars foreign rooms from the room scope. LANE-29 lets a seat pull such a post without a seat in the sending room, while PEER-02 serves segments only to principals with a seat, and OQ-41 covers only principal acts.",
    "Rule what an unnamed recall reaches of a foreign room, then align RCL-05, RCL-10, places.md Foreign room, hub relation 10 and PEER-02/OQ-41.",
    "decide", "hub#2 and places#6 each offer 'state the exception' or 'exclude'; places#6 adds a third option, narrowing the Foreign room definition.",
    ["State the exceptions: the run scope keeps the calling run's own events and the room scope the cross-room posts a room of its principal shows, both marked untrusted; RCL-10: 'Recall MUST extend to a foreign room only through a room parameter naming it in that call, beyond the calling run's own events and the cross-room posts a room of its principal shows, which recall marks untrusted'; places.md Foreign room says the same; fold serving cross-room posts into OQ-41 or PEER-02.",
     "Exclude: RCL-05's run scope drops writers of seats whose room has turned foreign, and a cross-room post from a foreign sending room is shown only when that room is named; places.md Foreign room says so.",
     "Narrow the definition: Foreign room '... other than only as a blind peer or only for the cross-room posts a room of its principal shows', plus option 1's run-scope wording in RCL-10."],
    True, [S+"05-functional-requirements.md", M+"places.md", M+"index.md", S+"05d-peer-requirements.md", S+"13-open-questions-and-risks.md"]))

A(c("No interface can write a directed post",
    ["pins-and-context#4"], ["pins-and-context", "hub"],
    [S+"09-interfaces.md:57", S+"09a-command-line-interface.md:56", S+"05b-lane-requirements.md:24"],
    "LANE-12 and the model's Directed post need a post directed to one agent, but neither `room_post` nor `parinn room post` can direct one. That breaks VIEW-03's rule that everything the room view does exists in the CLI or MCP.",
    "Add a target parameter to `room_post` and `parinn room post`, named by the hub's option rule.",
    "decide", "The reviewer asks the stakeholder to confirm the parameter name.",
    ["`seat` (optional: directs the post to that run seat's agent, LANE-12) on `room_post`; `parinn room post <room> [--seat <seat>]`.",
     "`run` / `--run` naming the target run; ParleyInn resolves its seat in the room."],
    False, [S+"09-interfaces.md", S+"09a-command-line-interface.md"]))

A(c("The LANE-33 scenario caps room_summary_request, which no requirement caps",
    ["pins-and-context#5"], ["pins-and-context"],
    [F+"lane.feature:515-516", S+"09-interfaces.md:73-74", S+"09-interfaces.md:175", S+"05b-lane-requirements.md:45"],
    "LANE-33, §9.2 and §9.6 cap only `room_summary_get` by `room_summary.max_model_tokens`. The scenario asks `room_summary_request` for 2000 model tokens and asserts the request is recorded 'for 500 model tokens at most', a cap no requirement states.",
    "Make the SRS and the scenario agree.",
    "decide", "",
    ["SRS §9.2 `room_summary_request`: '`size` (model tokens wanted, capped by `room_summary.max_model_tokens`)'; §9.6 key description: 'the agent's principal's cap on `room_summary_request` and `room_summary_get`'; LANE-33 to match.",
     "Change the scenario to assert the cap on `room_summary_get` only."],
    False, [S+"09-interfaces.md", S+"05b-lane-requirements.md", F+"lane.feature"]))

A(c("Working view is defined but the SRS never uses it; it says 'the model's context' instead",
    ["pins-and-context#11"], ["pins-and-context"],
    [M+"pins-and-context.md:141", S+"05b-lane-requirements.md:36", S+"05b-lane-requirements.md:42", S+"05b-room-view-requirements.md:52", S+"05b-room-view-requirements.md:54", S+"05c-principal-and-peer-requirements.md:41", S+"06-security.md:19", S+"06-security.md:100", S+"09-interfaces.md:87"],
    "The model's Working view ('whatever is currently in the model's context window') appears nowhere in the SRS or scenarios. The same meaning appears as 'the model's context' or 'an agent's context' in at least eight places. One concept therefore has two names.",
    "Use one name.",
    "decide", "",
    ["SRS says 'working view' at each listed place (e.g. LANE-24 'MUST NOT enter any working view').",
     "Model's Working view entry states that 'the model's context' is the harness's phrase for it, and the SRS keeps that phrase."],
    False, [S+"05b-lane-requirements.md", S+"05b-room-view-requirements.md", S+"05c-principal-and-peer-requirements.md", S+"06-security.md", S+"09-interfaces.md", M+"pins-and-context.md"]))

A(c("INJ-05 injects 'qualifying pins only' when the run is ambiguous, but qualifying pins are defined per run",
    ["pins-and-context#12"], ["pins-and-context", "principals-and-agents"],
    [S+"05-functional-requirements.md:109", F+"restore.feature:52-60"],
    "Pin and PIN-10 define a run's qualifying pins by the rooms the run has had a seat in. With the run ambiguous that set is undefined, and pins of a room only another candidate run had a seat in would breach PIN-10. The scenario does not decide it.",
    "Rule which pins restore under ambiguity, then reword INJ-05 and its scenario.",
    "decide", "",
    ["'... it MUST inject only the qualifying pins of rooms every candidate run has had a seat in, at least its personal room.'",
     "Inject only the personal room's qualifying pins.",
     "Inject no pins and state the ambiguity in the restore block (audited)."],
    True, [S+"05-functional-requirements.md", F+"restore.feature"]))

A(c("Visibility 'private' and 'shared' constrain nothing but publishing and blind peers",
    ["places#7"], ["places"],
    [M+"places.md:58-61", S+"05b-lane-requirements.md:29", S+"05d-peer-requirements.md:18", S+"05d-peer-requirements.md (PEER-08)"],
    "Only publishing and blind-peer enrollment check visibility. Nothing says whether an invite, admission change or join into a non-personal private room is refused or makes it shared; PEER-02 serves and PEER-08's git carrier stores segments whatever the visibility. A room can therefore show 'private' while other principals hold it.",
    "Rule what private and shared mean, then edit the model's Visibility and LANE-17.",
    "decide", "",
    ["'private' and 'shared' are derived from the room's principals and shown, never set; Visibility's settable values are published and stored on blind peers.",
     "'private' gates invites, admission and joins of other principals (refused and audited while private), and PEER-02/PEER-08 serve and store per visibility."],
    True, [M+"places.md", S+"05b-lane-requirements.md", S+"05d-peer-requirements.md"]))

A(c("PEER-05 conditions on 'a node marked ephemeral', a mark nothing defines",
    ["places#9"], ["places", "seats-and-keys"],
    [S+"05d-peer-requirements.md:21", M+"places.md:17"],
    "The model's ephemeral node is only 'one in a short-lived cloud environment'. No requirement or model entry defines the mark PEER-05 conditions on, so the requirement cannot be decided.",
    "Rule what makes a node ephemeral; the model's Node names it and PEER-05 reads 'An ephemeral node MUST ...'.",
    "decide", "",
    ["A node is ephemeral when its seats are certified by a token key under an access token (PEER-06).",
     "A settings key (`node.ephemeral`) the node's principal or managed policy sets."],
    False, [S+"05d-peer-requirements.md", M+"places.md"]))

A(c("Managed policy certifies service accounts by listing them, outside the record and the key set",
    ["principals-and-agents#1"], ["principals-and-agents", "seats-and-keys"],
    [M+"principals-and-agents.md:16-18", M+"principals-and-agents.md:24-26", M+"principals-and-agents.md:30-33", S+"05-functional-requirements.md:55", M+"seats-and-keys.md:75-76", M+"record.md:11-14"],
    "Managed policy 'revokes one it lists by no longer listing it', but a listing is a settings layer beside the record, not part of the key set, and no requirement records it. A delisted key then counts as never certified, a person's, contradicting 'a service account whose certificate is revoked stays a service account' (Principal, OWN-27). Nodes under different managed policies derive verdict acceptance and trust-grant refusal differently, breaking LANE-22, LANE-31, PRV-02 and I10's inputs.",
    "Rule how a managed-policy listing enters the derivation.",
    "decide", "",
    ["PRV-10 and the model (Managed policy, Service account, seats-and-keys Key set): each managed-policy listing and delisting is recorded as a signed certificate or revocation in the node's key set and replicates like one, so a delisted account stays a service account.",
     "Drop managed policy as a certifier: model, PRV-10, OWN-27, OWN-29, 09a row 61, provenance.feature:214, principal-acts.feature:330 and :364."],
    True, [M+"principals-and-agents.md", M+"seats-and-keys.md", S+"05-functional-requirements.md", S+"05c-principal-and-peer-requirements.md", S+"09a-command-line-interface.md", F+"provenance.feature", F+"principal-acts.feature"]))

A(c("The model says a harness clear starts a new run; no requirement or assumption backs it",
    ["principals-and-agents#6"], ["principals-and-agents", "harness-facts"],
    [M+"principals-and-agents.md:44-47", S+"05-functional-requirements.md:17", S+"02-context.md:49"],
    "Run says 'a harness resume that starts a new harness session, or a harness clear, starts a new run'. REC-02 and ASM-01 cover only resume, and the term is used nowhere in the SRS. Run is keyed by harness session, so if a clear kept its session_id the model would contradict itself.",
    "Back the clause with an assumption and a requirement, or drop it.",
    "decide", "",
    ["ASM-01 adds 'a SessionStart with source = clear carries a new session_id'; REC-02 adds 'A harness clear MUST start a new run'; add a step to REC-02's scenario and an assumptions.feature row.",
     "Drop 'or a harness clear' from the model's Run."],
    False, [M+"principals-and-agents.md", S+"05-functional-requirements.md", S+"02-context.md", F+"record.feature", F+"assumptions.feature"]))

A(c("Receipts: SEC-19 says a purge receipt carries only digests, and which receipts are signed is unsettled",
    ["record#8"], ["record"],
    [S+"06-security.md:109", S+"06-security.md:121", S+"06b-boundary-register.md:23", M+"record.md:158-162"],
    "SEC-19 and §6.3 say 'nor is a head or purge receipt, which carries only digests (VIEW-10)'. SEC-31 makes the purge receipt state the scope, ranges, copies erased and every copy ParleyInn cannot erase, which is not only digests. The model calls every receipt signed, which VIEW-10 and SEC-27 do not require of a head receipt.",
    "Fix SEC-19's description and rule which receipts are signed.",
    "decide", "",
    ["SEC-19 and 06b:23: 'nor is a head receipt, which carries only digests (VIEW-10), or a purge receipt, which carries only ids, addresses, counts and the names of the copies it lists (SEC-31)'; VIEW-10/SEC-27 require the head receipt to be signed, so the model stands.",
     "Same SEC-19 wording; the model's Receipt narrows 'signed' to the purge receipt.",
     "Give the purge receipt its own §6.3 row instead of the SEC-19 sentence."],
    False, [S+"06-security.md", S+"06b-boundary-register.md", S+"05b-room-view-requirements.md", M+"record.md"]))

A(c("The Notification hook row cites ingestion under REC-13, which does not cover it",
    ["record#9"], ["record", "harness-facts"],
    [S+"09-interfaces.md:27", S+"05-functional-requirements.md (REC-13)", F+"record.feature (REC-13 scenario)"],
    "Ingest is reading a transcript into the record. REC-13 and its scenario list only PostToolUse, Stop, SubagentStop and PermissionRequest, and the Notification row's inputs carry no `transcript_path`. So 'ingestion (REC-13)' cites a requirement that does not cover an act that is not ingest.",
    "Say what the Notification handler does.",
    "decide", "",
    ["Row: 'None; records the notification text as `harness_text` (PRV-08)'.",
     "Add Notification to REC-13 and its scenario's Examples."],
    False, [S+"09-interfaces.md", S+"05-functional-requirements.md", F+"record.feature"]))

A(c("A rotated seat key gets no seat certificate, and device-key rotation's effect on seat certificates is unsaid",
    ["seats-and-keys#7"], ["seats-and-keys"],
    [S+"06-security.md:117", S+"05-functional-requirements.md:56"],
    "SEC-27 keeps a seat's id and writer across a seat-key rotation, signed by the old and new keys. PRV-11 certifies seat keys only 'when the seat starts', and under the model a key chains only through device, token or seat certificates, never through a rotation event. Without a new certificate a LANE-25 bar may not cover the new key (T26) and SEC-25 or PEER-07 may refuse its segments.",
    "Rule how a rotated seat key chains, and whether seat certificates survive a device-key rotation; the model's Seat key follows.",
    "decide", "",
    ["SEC-27 adds 'the new seat key MUST carry a seat certificate from the node's device key, or its token key, naming the same room and seat kind; seat certificates a rotated device key made stay valid until revoked'.",
     "The rotation event itself extends the old key's chain to the new key; the model's chaining rule adds rotation events."],
    True, [S+"06-security.md", S+"05-functional-requirements.md", M+"seats-and-keys.md"]))

A(c("Device scope covers acts, posts and pins, but requirements use it to limit reading",
    ["seats-and-keys#8"], ["seats-and-keys", "acts-and-roles"],
    [M+"seats-and-keys.md:50-52", S+"05-functional-requirements.md:55", S+"05c-principal-and-peer-requirements.md (OWN-16)", S+"05b-lane-requirements.md:28", M+"acts-and-roles.md:110-111"],
    "The model's device scope is 'the kinds of principal act it may sign, and of post and pin its seats may write'. PRV-10, OWN-16, LANE-16 and the Role entry all use device scope to limit what a paired phone reads. Nothing defines what a paired phone may read.",
    "Make the definition and its uses agree.",
    "decide", "",
    ["Model: 'a device scope (what it may read, the kinds of principal act it may sign, and of post and pin its seats may write)'.",
     "LANE-16 and Role drop 'within its device scope' after 'reading'; PRV-10/OWN-16 limit reading by their own words."],
    False, [M+"seats-and-keys.md", S+"05b-lane-requirements.md", M+"acts-and-roles.md"]))

A(c("PIN-11's reason 'its seat certificate or token key does not cover the pin's room' looks impossible",
    ["seats-and-keys#11"], ["seats-and-keys", "pins-and-context"],
    [S+"05-functional-requirements.md:72", F+"pins.feature:150"],
    "A pin's room is always its author seat's room (LANE-01), and the seat certificate's scope is the same on every node (PRV-02), so a pin cannot qualify on another of the principal's nodes and fail here for that reason. A seat outside a token key's scope is refused outright (peer.feature:67). The example at pins.feature:150 looks impossible to set up.",
    "State the case PIN-11 means, or drop it.",
    "decide", "",
    ["PIN-11: '... because this node does not hold the seat certificate or token certificate it chains through, a certificate it chains through is revoked or its event has not arrived ...'; pins.feature:150 to match.",
     "Drop that reason from PIN-11 and the pins.feature:150 example."],
    False, [S+"05-functional-requirements.md", F+"pins.feature"]))

A(c("`recall.default_scope` lets configuration change the default recall scope RCL-05 fixes",
    ["trust-and-flow#6"], ["trust-and-flow"],
    [S+"09-interfaces.md:182", S+"05a-administration-requirements.md:17", S+"06-security.md:101", F+"security.feature:144", S+"05-functional-requirements.md:82"],
    "Recall scope defaults to the current run and extends only by an explicit, logged call (RCL-05). The settings key `recall.default_scope` (repository configuration may set only `run`), ADM-04 ('loosens ... recall scope'), SEC-11 and security.feature:144 let the principal's configuration change that default.",
    "Remove the loosening, or make RCL-05 allow it.",
    "decide", "",
    ["Drop `recall.default_scope` from §9.6, and 'recall scope' from ADM-04's loosening list; SEC-11 and security.feature:144 to match.",
     "Keep the key but fix it to `run` everywhere (and drop it from ADM-04).",
     "Reword RCL-05 to allow a configured default, logged and shown on Health."],
    True, [S+"09-interfaces.md", S+"05a-administration-requirements.md", S+"06-security.md", F+"security.feature", S+"05-functional-requirements.md"]))

A(c("§9.7.6 shows unsandboxed agents as one Health line; OWN-22 wants the risk acceptance beside each run on every surface",
    ["trust-and-flow#7"], ["trust-and-flow", "places"],
    [S+"09b-lane-vocabulary.md:136-137", S+"05c-principal-and-peer-requirements.md:38", F+"principal-acts.feature:269"],
    "§9.7.6 says 'Unsandboxed agents are one line on Health, not a mark on every tile'. OWN-22 and its scenario say 'The risk acceptance MUST be shown on every surface beside each run that relies on it'.",
    "Choose one.",
    "decide", "",
    ["§9.7.6: 'Unsandboxed agents are one line on Health, and each run that relies on a risk acceptance shows it beside the run (OWN-22)'.",
     "Narrow OWN-22 and its scenario to the Health line."],
    False, [S+"09b-lane-vocabulary.md", S+"05c-principal-and-peer-requirements.md", F+"principal-acts.feature"]))

A(c("Room tools return room content with only a `room` parameter, not RCL-05's explicit, logged scope",
    ["trust-and-flow#8"], ["trust-and-flow"],
    [S+"09-interfaces.md:50", S+"09-interfaces.md:56", S+"09-interfaces.md:74", S+"05-functional-requirements.md:82"],
    "RCL-05 requires an explicit `scope` parameter, logged, to extend recall past the run. `room_get` with `id`, `pin_list` with `room` and `room_summary_get` return room content given only `room`.",
    "State in §9.2 how these tools relate to RCL-05.",
    "decide", "",
    ["§9.2: naming `room` in `room_get`, `pin_list` and `room_summary_get` is an explicit extension under RCL-05, logged as a recall event (RCL-07), limited to the principal's rooms the agent has a seat in (a foreign room only under RCL-10).",
     "These tools take RCL-05's `scope` parameter like the recall tools."],
    False, [S+"09-interfaces.md", S+"05-functional-requirements.md"]))

A(c("The model's Sandbox state includes a policy digest no requirement mentions",
    ["trust-and-flow#12"], ["trust-and-flow"],
    [M+"trust-and-flow.md:117-119", S+"05c-principal-and-peer-requirements.md:38"],
    "Sandbox state is 'What confines a run, its policy digest, and which residual risks it blocks' and cites OWN-22, which requires no policy digest; nothing in the SRS mentions one.",
    "Align the model and OWN-22.",
    "decide", "",
    ["Drop 'its policy digest' from the model's Sandbox state.",
     "Add to OWN-22 that the sandbox state records the sandbox policy's digest."],
    False, [M+"trust-and-flow.md", S+"05c-principal-and-peer-requirements.md"]))

A(c("'Origin' is used in the browser's sense beside the model's Origin of an event",
    ["whole#11"], ["hub", "record", "components-and-surfaces"],
    [M+"record.md:78", M+"components-and-surfaces.md:35", S+"06-security.md:56", S+"06-security.md:110", S+"06-security.md:111", S+"05c-principal-and-peer-requirements.md:27", S+"05c-principal-and-peer-requirements.md:32", S+"05b-room-view-requirements.md:59", S+"13-open-questions-and-risks.md:45", M+"index.md:15"],
    "The model defines Origin as how an event reached the record, yet components-and-surfaces.md:35 and the SRS (SEC-20, SEC-21, OWN-11, OWN-16, VIEW-22, T15, OQ-38 and their scenarios) use 'origin' unqualified in the browser's sense. The hub's list of outside things that keep their own names does not include it.",
    "Qualify the browser's sense.",
    "decide", "",
    ["Add 'a web origin' to the hub's outside-things list (index.md:15) and say 'web origin' in components-and-surfaces.md:35, the SRS rows listed and their scenarios.",
     "State in record.md's Origin entry that the browser's origin keeps its own sense; no other text changes."],
    False, [M+"index.md", M+"components-and-surfaces.md", M+"record.md", S+"06-security.md", S+"05c-principal-and-peer-requirements.md", S+"05b-room-view-requirements.md", S+"13-open-questions-and-risks.md", F+"security.feature", F+"principal-acts.feature"]))
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
# Part 3: ids, cross conflicts, instruction edits, validation, output.
import json, os

OUT = os.path.dirname(os.path.abspath(__file__))
findings = json.load(open(os.path.join(OUT, "all-findings.json")))
all_rids = [f["rid"] for f in findings]
htr_flag = {f["rid"]: f["hard_to_revert"] for f in findings}

order = {"stakeholder": 0, "decide": 1, "fix": 2}
CLUSTERS.sort(key=lambda x: order[x["cls"]])  # stable within class
out_clusters = []
by_rid = {}
for i, cl in enumerate(CLUSTERS, 1):
    cid = "C%02d" % i
    d = {"id": cid, "title": cl["title"], "rids": cl["rids"], "owners": cl["owners"],
         "locations": cl["locations"], "problem": cl["problem"], "proposed_fix": cl["proposed_fix"],
         "class": cl["cls"], "conflict": cl["conflict"], "options": cl["options"],
         "hard_to_revert": cl["hard_to_revert"], "touches": cl["touches"]}
    out_clusters.append(d)
    for r in cl["rids"]:
        by_rid.setdefault(r, []).append(cid)

def cid(rid):
    return by_rid[rid][0]

# hard_to_revert sanity: true only if some rid flagged it
for d in out_clusters:
    flagged = any(htr_flag[r] for r in d["rids"])
    assert not (d["hard_to_revert"] and not flagged), d["id"]

CROSS = [
    (["acts-and-roles#1", "seats-and-keys#8", "acts-and-roles#5"],
     S+"05b-lane-requirements.md:28 (LANE-16) and "+M+"acts-and-roles.md:110-115 (Role)",
     "The role-default cluster rewrites LANE-16's 'Except the owner's device seats ... MUST be a viewer, but ... contributor role, and a paired phone's personal-room seat none, reading within its device scope' sentence and the matching Role sentences; the device-scope cluster's option 2 deletes 'within its device scope' from that same sentence and from Role; the owner-has-no-role fix edits LANE-16's last sentence. Compatible, but apply in one change to one merged sentence.",
     'yes: merge into one LANE-16 sentence and one Role sentence'),
    (["harness-facts#8", "principals-and-agents#3", "principals-and-agents#4"],
     S+"invariants.md:37-44 (I2 closed paths) and "+M+"principals-and-agents.md:18-19",
     "Both stakeholder clusters reword adjacent clauses of I2's closed-path list; do them under one ADR and one security review (ENG-29), then one `mdsmith fix .` for CLAUDE.md. The model's Principal entry fix must use the same delegation wording I2 ends up with.",
     "yes: one ADR, one security review; C74 follows C02's wording"),
    (["hub#2", "trust-and-flow#6", "trust-and-flow#8", "whole#2"],
     S+"05-functional-requirements.md:82 (RCL-05), :87 (RCL-10), :47 (PRV-02)",
     "Three decide clusters edit RCL-05: the foreign-room one its last sentence (and RCL-10), recall.default_scope its option 3 the default sentence, the room-tools one the extension sentence. The PRV-02 fix adds 'whether the event's room is foreign' as an input; the foreign-room cluster's option 3 narrows what 'foreign' means there. Rule the three together and edit RCL-05 once.",
     "depends on options: C13 option 3 changes what 'foreign' means in C106's new PRV-02 input; rule C13, C27 and C29 together"),
    (["record#4", "components-and-surfaces#7"],
     M+"record.md:152-157 (Integrity status)",
     "The per-event integrity status fix rewrites the entry's first sentence; the 'UI' fix rewrites its last sentence ('The UI never says \"secure\"'). Compatible; same entry.",
     'yes'),
    (["record#6", "record#14"],
     M+"record.md:75-76",
     "Both edit the one sentence listing what is `structural`: 'key rotations' -> 'seat-key rotations' and 'a witness check's record' -> 'the event recording a witness check'. Compatible; apply together.",
     'yes'),
    (["record#10", "seats-and-keys#4"],
     S+"06b-boundary-register.md:58 (row 27)",
     "Same cell. Merged text: 'stores and serves sealed ranges encrypted to the device keys of the room's principals, or a token-key-only node's token key; reads no content'.",
     'yes: merged text given'),
    (["seats-and-keys#1", "seats-and-keys#2", "principals-and-agents#2"],
     S+"05b-lane-requirements.md:35 (LANE-23)",
     "Three fixes edit different sentences of LANE-23 (seat id derivation, seat-certificate hedge, a new personal-room leave/kick refusal). Compatible; one change avoids merge churn in the wide table row.",
     'yes'),
    (["components-and-surfaces#1", "whole#11", "places#3"],
     S+"05c-principal-and-peer-requirements.md:27 (OWN-11), :32 (OWN-16)",
     "The `parinn ui --phone` cluster's option (b) rewrites OWN-16's issuing sentence, which the 'origin' cluster's option 1 also edits ('web origin'); the authenticator fix and 'web origin' both edit OWN-11. Compatible if the origin rename is applied last.",
     "yes, if C31's rename is applied after C05 option (b) rewrites OWN-16"),
    (["components-and-surfaces#7", "whole#11"],
     S+"06-security.md:56 (T15)",
     "'Local web attack on the UI' -> 'on the browser room view' in the threat column; 'web origin' in the description column. Compatible.",
     'yes'),
    (["record#8", "git-and-forge#8"],
     S+"06-security.md:109 (SEC-19)",
     "Different sentences of SEC-19: the receipts sentence and the CI 'build-time reach evidence' sentence. Compatible.",
     'yes'),
    (["harness-facts#1", "places#10"],
     S+"05b-lane-requirements.md:15 (LANE-02)",
     "The hook-event cluster's option 1 rewrites 'at the first hook event'; the git-clone fix rewrites 'clone' twice in the same row. Compatible.",
     'yes'),
    (["git-and-forge#1", "git-and-forge#2", "whole#8"],
     S+"05c-principal-and-peer-requirements.md:37 (OWN-21), :43 (OWN-27) and "+M+"git-and-forge.md (Branch, tree)",
     "The branch-head cluster's option 1 widens OWN-21/OWN-27 to the worktree's tree, which the tree-definition fix makes a defined term; the OWN-27 'ParleyInn records no verdict' fix edits another sentence of the same row. Compatible; option 1 relies on the tree fix landing first.",
     'yes; C08 option 1 needs C45 to land first'),
    (["trust-and-flow#7", "trust-and-flow#12"],
     S+"05c-principal-and-peer-requirements.md:38 (OWN-22)",
     "The Health-line cluster's option 2 narrows OWN-22's display sentence; the policy-digest cluster's option 2 adds a clause to OWN-22. Different sentences; compatible.",
     'yes'),
    (["pins-and-context#11", "pins-and-context#8"],
     S+"03-rationale.md:13 and the Working view entry",
     "The §3 fix uses the name 'working view'; it assumes the Working view entry keeps its name under either option of the working-view cluster.",
     'yes, while Working view keeps its name'),
    (["git-and-forge#13", "trust-and-flow#15"],
     M+"index.md catalog (generated)",
     "Both change a concept file's front-matter summary; one `mdsmith fix` regenerates the hub catalog for both.",
     'yes'),
]
cross = []
for rids, loc, note, compatible in CROSS:
    ids = []
    for r in rids:
        if cid(r) not in ids:
            ids.append(cid(r))
    cross.append({"clusters": ids, "location": loc, "note": note, "compatible": compatible})

SPEC = ["acts-and-roles", "components-and-surfaces", "git-and-forge", "harness-facts", "pins-and-context",
        "places", "principals-and-agents", "record", "seats-and-keys", "trust-and-flow"]
ALL_SPEC = [".claude/agents/domain-model-%s.md" % s for s in SPEC]
HUB = ".claude/agents/domain-model-hub.md"
WHOLE = ".claude/agents/domain-model.md"

IE = []
def ie(target, section, edit, rationale, sources, needs_ruling=False):
    # `edit` is the text to put into the agent file: it must name no model concept,
    # relation or excluded term (template preamble; domain-model.md, "When the model changes" step 5).
    # `rationale` may name them; it is not inserted.
    IE.append({"target": target, "section": section, "edit": edit, "rationale": rationale,
               "from": sources, "needs_stakeholder_ruling": needs_ruling})

ie(ALL_SPEC, "How you review, step 1",
   "Replace 'then each concept file your file points to' with 'then every concept file whose concepts your file uses; the concept files seldom link to one another, so when in doubt read them all'.",
   "git-and-forge.md, record.md and places.md have no links to other model files; each reviewer read all of them anyway.",
   ["git-and-forge", "record", "places"])
ie(ALL_SPEC + [HUB, WHOLE], "How you review (new first line)",
   "Add: 'The caller's scope wins. When the caller names files, sections or a diff, search only inside them, even where a step below says to search the repository, and say what you left out.'",
   "The hub template's step 4 says 'search the whole repository'; this round was scoped to the SRS and scenarios.",
   ["hub"])
ie(ALL_SPEC + [HUB, WHOLE], "How you review (skip list)",
   "Add: 'Skip what the hub's section on terms that are not ParleyInn concepts exempts: historical records, the SRS change log and the Status row in docs/srs/index.md included, and files an outside tool writes and maintains.'",
   "The whole-model reviewer searched docs/srs/index.md's change log, which the hub already treats as a historical record.",
   ["whole", "hub"])
ie(ALL_SPEC + [HUB, WHOLE], "How you review (words that coincide with the model's)",
   "Add: 'The hub gives every word one meaning. A word the model defines, used in its ordinary English or another domain's sense, is a finding wherever it appears in the SRS, the scenarios, the model or an instruction file, normative or not, unless the hub's rule on outside things qualifies it or the hub lists the use as allowed. Report every such use of one word as one grouped needs-fix finding, with each place and a plain replacement.'",
   "Asked by four reviewers (concurrent, check, evidence, result, tree, pinned, recall, idle, scope, account). The edit applies the hub's existing rule (index.md:13). Ask the stakeholder whether non-normative rationale prose (§3, §4 notes, §11) should get a laxer rule; C54 is the test case.",
   ["acts-and-roles", "git-and-forge", "pins-and-context", "whole"], True)
ie(ALL_SPEC, "How you report (kinds)",
   "Replace 'Outside a change to your file, report a gap or contradiction inside it as a question for the stakeholder' with: 'Kinds: needs-fix is one obvious wording fix in any file, your own included, such as a citation naming the wrong requirement, or a sentence of your file saying more or less than the requirement it cites, with nothing new defined; question is a gap or contradiction that needs a ruling, where either side could move or something new must be defined; other-specialist is a fix that lands wholly in a file another specialist owns, which you name. A finding whose fix spans your file and another's is yours: file it once and name the other specialist in breaks.'",
   "record (items 11, 12) and seats-and-keys (item 10) filed plain own-file citation errors as questions; components-and-surfaces and places could not tell question from other-specialist for cross-file findings; principals-and-agents asked the same for invariant wording.",
   ["record", "seats-and-keys", "components-and-surfaces", "places", "principals-and-agents"])
ie(ALL_SPEC, "How you report (which side moves)",
   "Add: 'When the SRS contradicts your file: if the rest of the SRS and the scenarios carry your file's meaning, the departing text is needs-fix; if your file says more than, or other than, the requirement it cites and the SRS is right, your file is needs-fix; if either side could move, it is a question.'",
   "places used needs-fix where the model's meaning was clearly intended and question where either side could move; this makes that the rule.",
   ["places"])
ie(ALL_SPEC, "How you review, step 6",
   "Extend: 'A requirement's invariant trace (its Traces column, its scenario's tags and Appendix B) is in scope where it stretches an invariant your concepts serve, and so are the assumption rows, spikes and open questions (§2.3, §13) a requirement on your concepts is hedged on. A defect in an invariant's wording sets invariant: true, with kind needs-fix when the wording is unambiguous and question when it needs a ruling; either way it needs an ADR and a security review (ENG-29).'",
   "harness-facts reported the LANE-28 trace to I7 (C12) and PIN-07's wrong assumption gate (C11) without knowing they were in scope.",
   ["harness-facts", "principals-and-agents"])
ie(ALL_SPEC, "How you review (scenarios)",
   "Add: 'A scenario that leaves out a precondition one of your concepts decides is needs-fix. Drift between a requirement and its scenario about one of your concepts is in scope even when no word is wrong, and so is a @pending scenario that skips a clause about one. For a scenario without @pending, name its step binding in cmd/parinn/bdd_<section>_test.go and any drift case under internal/drift that quotes it; the fix changes them in the same change.'",
   "Role defaults omitted in two scenarios (C03); ENG-29 vs its scenario (C99); ENG-01 is bound at cmd/parinn/bdd_engineering_test.go:40 (C66).",
   ["pins-and-context", "acts-and-roles", "places", "trust-and-flow"])
ie(ALL_SPEC, "How you review (unused concepts)",
   "Add: 'A concept your file defines that no requirement or scenario uses, or that the SRS names in other words, is a question: the SRS takes the model's name, or the concept goes.'",
   "Working view (C16).",
   ["pins-and-context"])
ie(ALL_SPEC, "How you review (delivery order)",
   "Add: 'A rule hedged on delivery order (\"before <requirement id> ships\") follows the milestone order in docs/srs/12-delivery-plan.md; read it before calling an unhedged rule a defect.'",
   "seats-and-keys findings 2 and 7 turn on 'before PRV-10 ships'.",
   ["seats-and-keys"])
ie(ALL_SPEC + [HUB, WHOLE], "How you review (what else to read)",
   "Add: 'Proposed ADRs under docs/adr may be read for context, never as normative. plan/ and research/ need not be read.'",
   "components-and-surfaces read ADR-2610042341 (proposed) to judge CON-03 (C07).",
   ["components-and-surfaces"])
ie(ALL_SPEC + [HUB], "How you report (finding fields)",
   "Add: '`file` holds one repository-relative path and `line` the first line; list the further places of a grouped repeat in `also`, an array of \"path:line\" strings.'",
   "components-and-surfaces split one repeat into three findings; principals-and-agents packed several paths into `file`.",
   ["components-and-surfaces", "principals-and-agents"])
ie(ALL_SPEC, "How you review, step 3",
   "Add after 'any word the hub excludes in its favour': 'that is, each row of the hub's table of terms that are not ParleyInn concepts whose replacement your file defines.'",
   "acts-and-roles could not tell which excluded terms point to its file. Optional companion edit to the hub (docs/domain-model/index.md, a model edit the domain-model agent reviews; no new concept): a 'Defined in' column. Mapping found this round — acts-and-roles: operator as a role, owner act, read only; components-and-surfaces: run component, trusted boundary ('boundary'), Needs you as a status, hide; git-and-forge: project, attempt, own run and witness run, judge, approval gate; harness-facts: session; pins-and-context: claim of work, unqualified token (model token); places: lane, room board, project; principals-and-agents: tenant, bound human, bot, co-author, actor, poster, child agent, participant and player (member), operator as a person, session (run); record: import of a transcript or a peer's segments, imported run, work marker, operator (provenance class); seats-and-keys: writer key, writer certificate, service-account seat, owner key, participant and player (seat), unqualified token (access token); trust-and-flow: owner surface, trusted boundary, away mode, room alias, presence check, Needs you as a run status (Asking).",
   ["acts-and-roles"])

WTS = {
    "git-and-forge": ("SRS: add docs/srs/05b-room-view-requirements.md, docs/srs/09a-command-line-interface.md, docs/srs/09b-lane-vocabulary.md. Scenarios: add features/lane-view.feature. Add an Invariants line, as every other specialist has, listing those the file's LANE and OWN rows trace.",
                      "VIEW-13, VIEW-19, VIEW-21, VIEW-22, §9.7.2 and §9.7.3 carry many of its concepts.", ["git-and-forge"]),
    "record": ("SRS: add docs/srs/08-data-and-storage.md, docs/srs/09-interfaces.md, docs/srs/09a-command-line-interface.md, docs/srs/09b-lane-vocabulary.md, docs/srs/05d-peer-requirements.md. Scenarios: add features/peer.feature, features/lane-view.feature.",
               "Store and payload store (§8), address forms and the envelope's integrity field (§9.2), export, import, purge, receipts and backup (§9.5), §9.7.5, segments and sealed ranges (PEER rows), VIEW-08/10/11.", ["record"]),
    "seats-and-keys": ("SRS: add docs/srs/05c-principal-and-peer-requirements.md, docs/srs/09-interfaces.md, docs/srs/09a-command-line-interface.md, docs/srs/12-delivery-plan.md. Scenarios: add features/principal-acts.feature, features/pins.feature.",
                       "OWN-11, OWN-16, OWN-17, OWN-29; key and token verbs and MCP parameters; the PRV-10 hedges.", ["seats-and-keys"]),
    "trust-and-flow": ("SRS: add docs/srs/09-interfaces.md, docs/srs/09a-command-line-interface.md, docs/srs/09b-lane-vocabulary.md. Invariants: add I8.",
                       "Run and room status, queue classes, trust marks, the focus set, the continuation cursor, recall.* and quota.* live there; RCL-05, RCL-10, OWN-26 and OWN-29 trace I8.", ["trust-and-flow"]),
    "places": ("SRS: add docs/srs/05d-peer-requirements.md. Scenarios: add features/peer.feature, features/lane-view.feature, features/pins.feature.",
               "Peer, blind peer, sync and ephemeral node are defined against PEER rows; Foreign room and Personal room are used in lane-view and pins scenarios.", ["places"]),
    "harness-facts": ("SRS: add docs/srs/09-interfaces.md, docs/srs/13-open-questions-and-risks.md, docs/srs/appendix-b-invariant-coverage.md.",
                      "Derived from where this round's findings landed: §9.1's hook table, OQ-02 and the spikes, and the traces of C12.", ["harness-facts"]),
    "pins-and-context": ("SRS: add docs/srs/09-interfaces.md, docs/srs/08-data-and-storage.md, docs/srs/05d-peer-requirements.md. Code: the step bindings in cmd/parinn/bdd_*_test.go.",
                         "Derived from this round: room_post and room_summary_* (§9.2, §9.6), §8.4, PEER-06, and the ENG-01 binding.", ["pins-and-context"]),
    "components-and-surfaces": ("SRS: add docs/srs/02-context.md, docs/srs/04-reference-architecture.md, docs/srs/09a-command-line-interface.md, docs/srs/12-delivery-plan.md.",
                                "Derived from where this round's findings landed (CON-03, §4.2, §9.5, §12).", ["components-and-surfaces"]),
    "acts-and-roles": ("SRS: add docs/srs/07-non-functional-requirements.md. Scenarios: add features/landmarks.feature.",
                       "Derived from where this round's findings landed (C03, C36).", ["acts-and-roles"]),
    "principals-and-agents": ("SRS: add docs/srs/02-context.md, docs/srs/09b-lane-vocabulary.md.",
                              "Derived from where this round's findings landed (ASM-01, durable volumes, §9.7.6).", ["principals-and-agents"]),
}
for s, (txt, why, src) in WTS.items():
    ie([".claude/agents/domain-model-%s.md" % s], "Where to start", txt, why, src)

ie([".claude/agents/domain-model-components-and-surfaces.md", HUB], "How you report (who owns a meaning)",
   "Add: 'The concept file that defines a concept owns its meaning; the hub owns only the form of names. Where the hub's naming section uses a concept's word in another sense, report it to the hub specialist as other-specialist.'",
   "The hub's Names section counts `ui` and `launch` among CLI commands while components-and-surfaces.md's Core excludes `parinn ui` from the CLI (C05).",
   ["components-and-surfaces"])
ie([".claude/agents/domain-model-harness-facts.md"], "How you review (an outside program's words)",
   "Add: 'An outside program's own identifiers keep their names under the hub's rule on outside things. A word of an outside program written as prose that a requirement relies on normatively needs a model entry or a rewording: report it as a question.'",
   "Hook output and hook event (C10). Settle together with C10's ruling.",
   ["harness-facts"], True)
ie([".claude/agents/domain-model-git-and-forge.md"], "How you review (an outside domain's terms)",
   "Add: 'An outside domain's own terms your file uses without defining them are outside things that domain qualifies. When a requirement binds behaviour to one, report as a question whether the model must define it.'",
   "git's head, base and remote URL; OWN-21's 'branch heads' (C08).",
   ["git-and-forge"])
ie([".claude/agents/domain-model-record.md"], "How you review (a thing used as an actor)",
   "Add: 'Where the model or the SRS makes one of your concepts the subject of an act it cannot perform, raise it once as a grouped question until the stakeholder rules; do not skip it silently.'",
   "A writer is a log, yet REC-06 ('assigned only by that writer'), ENG-09 and the model's Fork ('one writer sealed') make it an actor; the record reviewer accepted it without reporting.",
   ["record"], True)
ie([".claude/agents/domain-model-principals-and-agents.md", HUB], "How you review (unstated conventions)",
   "Add: 'Where text follows a convention the hub does not state, such as an informal stand-in for one of your concepts in scenario prose or a word in a requirement about the repository's own development process, report each convention once as a grouped question until the stakeholder rules.'",
   "Scenarios say 'the person' for the node's principal (SEC-19, PRV-04, PEER-01); ENG-21's 'a reviewer other than its author' sits beside Pull-request author. A ruling could add both to the hub's allowed uses.",
   ["principals-and-agents"], True)
ie([HUB], "How you review, step 3",
   "Add: 'A requirement that contradicts a relation is your finding, whether the relation cites a requirement id or names a concept.'",
   "hub#1 (LANE-01 vs relation 6, which cites '(Run seat)').",
   ["hub"])
ie([HUB], "How you review, step 5",
   "Widen to: 'Names: identifiers, commands, tool names and their parameters, settings keys, event names, the names requirements and scenarios give to counts, and crate and module names follow the hub's rules. The hub owns their form; the specialist of the concept a name carries owns its meaning.'",
   "recall_calls in a scenario (C59) and ENG-02's example crate (C111) fell between specialists.",
   ["hub"])
ie([HUB], "How you review, step 4",
   "Add: 'A third-party project's proper name cited as a source is an outside thing under the hub's rule, not a use of an excluded term.'",
   "'session-index README' at 02-context.md:53.",
   ["hub"])
ie([WHOLE], "How you review",
   "Add: 'In a standing review, also apply step 1 of \"When the model changes\" and report what it finds as a question.'",
   "The whole-model reviewer found C31 (origin) that way without being told to look.",
   ["whole"])

# validation
seen = [r for d in out_clusters for r in d["rids"]]
assert sorted(seen) == sorted(all_rids), (set(all_rids) - set(seen), set(seen) - set(all_rids))
assert len(seen) == len(set(seen)) == 129
for d in out_clusters:
    assert d["class"] in ("fix", "decide", "stakeholder")
    if d["class"] == "decide":
        assert 2 <= len(d["options"]) <= 3, d["id"]

bundle = {"clusters": out_clusters, "cross_conflicts": cross, "instruction_edits": IE}
with open(os.path.join(OUT, "bundle.json"), "w") as fh:
    json.dump(bundle, fh, indent=2, ensure_ascii=False)

# markdown
def esc(s):
    return s.replace("|", "\\|")
lines = ["# Blind review r33: bundled findings", "",
         "129 findings from 12 reviewers in %d clusters." % len(out_clusters), ""]
from collections import Counter
cnt = Counter(d["class"] for d in out_clusters)
lines += ["| Class | Clusters |", "| --- | --- |"]
for k in ("stakeholder", "decide", "fix"):
    lines.append("| %s | %d |" % (k, cnt[k]))
lines += ["", "## Clusters", "",
          "| Id | Class | Title | Owners | Rids | Hard to revert |",
          "| --- | --- | --- | --- | --- | --- |"]
for d in out_clusters:
    lines.append("| %s | %s | %s | %s | %s | %s |" % (
        d["id"], d["class"], esc(d["title"]), ", ".join(d["owners"]), ", ".join(d["rids"]),
        "yes" if d["hard_to_revert"] else ""))
lines += ["", "## Options for stakeholder and decide clusters", ""]
for d in out_clusters:
    if d["class"] == "fix":
        continue
    lines.append("### %s %s" % (d["id"], d["title"]))
    lines.append("")
    lines.append(d["problem"])
    lines.append("")
    for j, o in enumerate(d["options"], 1):
        lines.append("%d. %s" % (j, o))
    if d["conflict"]:
        lines += ["", "Conflict: " + d["conflict"]]
    lines.append("")
lines += ["## Cross-cluster overlaps", "", "No pair is incompatible under every option; these edit the same sentence or row and must be coordinated.", "", "| Clusters | Where | Note | Compatible |", "| --- | --- | --- | --- |"]
for x in cross:
    lines.append("| %s | %s | %s | %s |" % (", ".join(x["clusters"]), esc(x["location"]), esc(x["note"]), esc(x["compatible"])))
lines += ["", "## Instruction edits", "", "Edit text stays concept-free, as the templates require; examples sit in the Why column.", "", "| Target | Section | Edit | Why | From | Ruling |", "| --- | --- | --- | --- | --- | --- |"]
for e in IE:
    tgt = e["target"]
    if tgt == ALL_SPEC:
        t = "all ten specialists"
    elif tgt == ALL_SPEC + [HUB, WHOLE]:
        t = "all specialists, hub, domain-model"
    elif tgt == ALL_SPEC + [HUB]:
        t = "all specialists, hub"
    else:
        t = ", ".join(x.replace(".claude/agents/", "") for x in tgt)
    lines.append("| %s | %s | %s | %s | %s | %s |" % (esc(t), esc(e["section"]), esc(e["edit"]), esc(e["rationale"]), ", ".join(e["from"]),
                                                "needed" if e["needs_stakeholder_ruling"] else ""))
open(os.path.join(OUT, "bundle.md"), "w").write("\n".join(lines) + "\n")

print(dict(cnt), "clusters", len(out_clusters), "cross", len(cross), "edits", len(IE))
for d in out_clusters:
    if d["class"] != "fix":
        print(d["id"], d["class"], "|", d["title"])
