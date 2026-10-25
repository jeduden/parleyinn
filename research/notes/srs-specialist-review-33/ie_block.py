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
   "Add: 'A scenario that leaves out a precondition one of your concepts decides is needs-fix. Drift between a requirement and its scenario about one of your concepts is in scope even when no word is wrong, and so is a @pending scenario that skips a clause about one. For a scenario without @pending, name its step binding in cmd/parinn/bdd_<section>_test.go and any drift case under internal/drift that quotes it; the fix changes them in the same commit.'",
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
   "Add: 'In a standing review, also run step 1 of \"When the model changes\" and report what it finds as a question.'",
   "The whole-model reviewer found C31 (origin) that way without being told to look.",
   ["whole"])

