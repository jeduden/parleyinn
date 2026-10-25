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
