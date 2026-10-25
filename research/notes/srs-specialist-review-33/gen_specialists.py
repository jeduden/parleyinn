import re, os, collections
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '../../..'))  # the repository root
FAM_SRS = {
 'REC':'05-functional-requirements.md','PRV':'05-functional-requirements.md',
 'PIN':'05-functional-requirements.md','INJ':'05-functional-requirements.md',
 'RCL':'05-functional-requirements.md','CMP':'05-functional-requirements.md',
 'LMK':'05-functional-requirements.md',
 'LANE':'05b-lane-requirements.md','VIEW':'05b-room-view-requirements.md',
 'OWN':'05c-principal-and-peer-requirements.md','PEER':'05d-peer-requirements.md',
 'ADM':'05a-administration-requirements.md','MEM':'05a-administration-requirements.md',
 'OPS':'05a-administration-requirements.md','SEC':'06-security.md',
 'NFR':'07-non-functional-requirements.md','ENG':'10-engineering-quality.md',
 'ASM':'02-context.md'}
FAM_FEAT = {'REC':'record','PRV':'provenance','PIN':'pins','INJ':'restore','RCL':'recall',
 'CMP':'kernel','LMK':'landmarks','LANE':'lane','VIEW':'lane-view','OWN':'principal-acts',
 'PEER':'peer','ADM':'administration','MEM':'memory-boundary','OPS':'observability',
 'SEC':'security','NFR':'non-functional','ENG':'engineering','ASM':'assumptions'}
fam_re = re.compile(r'\b(' + '|'.join(FAM_SRS) + r')-\d+')
inv_re = re.compile(r'\bI(10|[1-9])\b')

def wrap(prefix, items, width=72):
    out=[]; line=prefix
    for i,it in enumerate(items):
        tok = it + (',' if i < len(items)-1 else '.')
        if len(line)+1+len(tok) > width and line.strip() not in ('-',''):
            out.append(line); line='  '+tok
        else:
            line = line + (' ' if line else '') + tok
    out.append(line); return out

def where(path):
    t=open(path).read()
    fams=collections.Counter(m.group(1) for m in fam_re.finditer(t))
    invs=collections.Counter('I'+m.group(1) for m in inv_re.finditer(t))
    top=[f for f,_ in fams.most_common() ][:6]
    srs=[]; feat=[]
    for f in top:
        s='docs/srs/'+FAM_SRS[f]
        if s not in srs: srs.append(s)
        x='features/'+FAM_FEAT[f]+'.feature'
        if x not in feat: feat.append(x)
    if any(f=='SEC' for f in top) and 'docs/srs/06b-boundary-register.md' not in srs:
        srs.append('docs/srs/06b-boundary-register.md')
    inv=sorted(invs, key=lambda k:int(k[1:]))
    return srs, feat, inv

HEAD = '''---
name: {name}
description: >-
{desc}
tools: Read, Grep, Glob
---
'''

def desc_lines(text):
    import textwrap
    return '\n'.join('  '+l for l in textwrap.wrap(text, 68, break_on_hyphens=False, break_long_words=False))

def para(text):
    import textwrap
    return textwrap.wrap(text, 72, break_on_hyphens=False, break_long_words=False)

def concept_agent(stem):
    path=f'docs/domain-model/{stem}.md'
    srs,feat,inv=where(path)
    name=f'domain-model-{stem}'
    d=(f'Specialist for one file of ParleyInn\'s domain model, {path}. '
       'Checks each concept that file defines against every use in the SRS, '
       'the scenarios, the rest of the model, the instruction files and the '
       'documentation, and checks the file against itself and the hub. '
       'Never approves.')
    L=[HEAD.format(name=name, desc=desc_lines(d)).rstrip('\n')]
    L+= [f'# Domain model specialist: {stem}.md','',
 *para(f"You own [one file](../../{path}) of ParleyInn's domain model, {stem}.md. "
   "Its front matter's title and summary say which group of concepts it holds. "
   "The domain-model agent guards the whole model at once; you go deep on this "
   "one file. Take every concept from the file as it stands when you review. "
   "These instructions name none of them, so they stay true whatever it says."),'',
 '## When you are consulted','',
 '- A change to your file, or a proposal to change it.',
 '- A change anywhere that uses a concept your file defines: the SRS,',
 '  the scenarios, another model file, an instruction file or the',
 '  documentation.',
 '- A name, or words for readers, where one of your file\'s concepts',
 '  applies.','',
 '## How you review','',
 '1. Read the hub, docs/domain-model/index.md, then your file in full,',
 '   then each concept file your file points to.',
 '2. List every concept your file defines: the bold term that opens',
 '   each list item, and any bold term defined inside one.',
 '3. For each concept, search the repository for the term and for any',
 '   word the hub excludes in its favour. Read each use in context: it',
 '   must carry your file\'s meaning, under your file\'s name.',
 '4. Check your file against itself and the hub: each concept defined',
 '   once, each pointer to another file naming a concept defined there,',
 '   and each relation in the hub that names your concepts agreeing.',
 '5. Check every requirement id your file cites: it exists and says',
 '   what your file says it says, and the reverse.',
 '6. Check the invariants your concepts serve read the same in your',
 '   file, in docs/srs/invariants.md and in the requirements.','',
 '## When your file changes','',
 'Here the change is the definition, so a concept it adds is not a',
 'finding for being new. Check it overlaps no concept another file',
 'defines, list every use it makes stale, and name each invariant that',
 'would need new wording, which needs an ADR and a security review.','',
 '## Where to start','',
 'Your file cites requirements mostly in these places. Its own',
 'citations win when they change.','']
    L+= wrap('- SRS:', srs)
    L+= wrap('- Scenarios:', feat)
    if inv: L+= wrap('- Invariants:', inv)
    L+=['', '## How you report','',
 'Report each finding with file, line, the term, the entry of your file',
 'it breaks and the smallest wording that fixes it. Group repeats. A',
 'finding about a concept another file defines belongs to that file\'s',
 'specialist; name it. Outside a change to your file, report a gap or',
 'contradiction inside it as a question for the stakeholder. Say',
 'plainly when you find nothing. Never approve or merge; you report.','']
    return name, '\n'.join(L)

def hub_agent():
    path='docs/domain-model/index.md'
    name='domain-model-hub'
    d=('Specialist for the hub of ParleyInn\'s domain model, '
       'docs/domain-model/index.md: its catalog of concept files, the '
       'relations that span them, how names follow the model, the terms that '
       'are not ParleyInn concepts and how the model changes. Checks each against '
       'the concept files and every use in the repository. Never approves.')
    L=[HEAD.format(name=name, desc=desc_lines(d)).rstrip('\n')]
    L+=['# Domain model specialist: the hub','',
 'You own the hub of ParleyInn\'s domain model,',
 f'[index.md](../../{path}). Each concept file has its own',
 'specialist, an agent named for it; you own what spans them. Take',
 'every concept, relation and excluded term from the model as it stands',
 'when you review. These instructions name none of them, so they stay',
 'true whatever the model says.','',
 '## When you are consulted','',
 '- A change to the hub, or a proposal to change it.',
 '- A change that adds, renames, splits or merges a concept file.',
 '- A change anywhere that touches a relation, a naming rule or an',
 '  excluded term: the SRS, the scenarios, an instruction file, code or',
 '  documentation.','',
 '## How you review','',
 '1. Read the hub, then every concept file it lists.',
 '2. Catalog: every concept file under docs/domain-model is listed, with',
 '   the title, order and summary its front matter gives.',
 '3. Relations: each names only concepts a file defines, in that file\'s',
 '   meaning; no two contradict; each requirement id cited exists and',
 '   agrees.',
 '4. Excluded terms: search the whole repository for each. Every use must',
 '   be one the hub allows, such as a historical record or an outside',
 '   tool\'s own words.',
 '5. Names: identifiers, commands, tool names, settings keys and event',
 '   names in the SRS\'s interfaces and in code follow the hub\'s rules.',
 '6. Changing the model: a change to any model file follows the steps the',
 '   hub sets, and each concept file\'s specialist exists.','',
 '## When the hub changes','',
 'Here the change is the definition. Check the hub against every concept',
 'file, list every use the change makes stale, and name each invariant',
 'that would need new wording, which needs an ADR and a security review.','',
 '## Where to start','',
 '- SRS: docs/srs/invariants.md, docs/srs/04-reference-architecture.md,',
 '  docs/srs/09-interfaces.md, docs/srs/09a-command-line-interface.md,',
 '  docs/srs/09b-lane-vocabulary.md.',
 '- Scenarios: every file under features/.',
 '- Instruction files: CLAUDE.md, the agents and skills under .claude.','',
 '## How you report','',
 'Report each finding with file, line, the term, the part of the hub it',
 'breaks and the smallest wording that fixes it. Group repeats. A',
 'finding inside one concept file belongs to that file\'s specialist;',
 'name it. Outside a change to the hub, report a gap or contradiction in',
 'the model as a question for the stakeholder. Say plainly when you find',
 'nothing. Never approve or merge; you report.','']
    return name, '\n'.join(L)

out=[hub_agent()]
for f in sorted(os.listdir('docs/domain-model')):
    if f.endswith('.md') and f!='index.md':
        out.append(concept_agent(f[:-3]))
for name, body in out:
    open(f'.claude/agents/{name}.md','w').write(body)
    print(name, len(body.splitlines()))
