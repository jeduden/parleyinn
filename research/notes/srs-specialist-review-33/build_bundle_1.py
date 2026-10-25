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
