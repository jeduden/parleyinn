"""Check annotated sources in ga/annotated/.

1. Removing every marker must reproduce the repository original byte for byte.
2. Markers balance and name known features.
3. For each optional feature F, removing only F's spans must leave no bold term
   that F's spans define mentioned elsewhere (dangling), reported per file.
usage: check_annotation.py [file ...]   (default: every annotated file)
Files not yet annotated are read from the repository, unmarked.
"""
import re, json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from assemble import strip, TOK
GA=os.path.dirname(os.path.abspath(__file__))
# the repository root; PARINN_REPO overrides it when this folder is copied elsewhere
REPO=os.environ.get('PARINN_REPO') or os.path.join(os.path.dirname(os.path.abspath(__file__)), '../../..')
SRC={f: f'{REPO}/docs/domain-model/{f}' for f in os.listdir(f'{REPO}/docs/domain-model') if f.endswith('.md')}
SRC['invariants.md']=f'{REPO}/docs/srs/invariants.md'
feats=json.load(open(f'{GA}/features.json'))
ids={f['id'] for f in feats['features']}|{'core'}

def load(name):
    for p in (f'{GA}/annotated/{name}.ann', f'{GA}/annotated/{name}'):
        if os.path.exists(p): return open(p).read()
    return open(SRC[name]).read()

def main(only, noorig=False):
    texts={n: load(n) for n in SRC}
    ok=True
    for n,t in texts.items():
        if only and n not in only: continue
        bad=[m.group(1) for m in TOK.finditer(t) if m.group(1) and m.group(1) not in ids]
        if bad: ok=False; print(f'{n}: unknown feature ids {sorted(set(bad))}')
        try: strip(t, ids)
        except ValueError as e: ok=False; print(f'{n}: {e}'); continue
        plain=TOK.sub('', t)
        if not noorig and plain!=open(SRC[n]).read():
            ok=False
            a=plain.split('\n'); b=open(SRC[n]).read().split('\n')
            for i,(x,y) in enumerate(zip(a,b)):
                if x!=y: print(f'{n}:{i+1}: text differs from original\n  got: {x!r}\n  want: {y!r}'); break
            else: print(f'{n}: length differs ({len(a)} vs {len(b)} lines)')
    # dangling per feature
    for f in sorted(ids-{'core'}):
        keep=ids-{f}
        removed=[]; kept={}
        for n,t in texts.items():
            try: k,r=strip(t, keep)
            except ValueError: continue
            kept[n]=k; removed+=r
        terms=sorted({r for r in removed if len(r.strip('`'))>=3})
        keptbold={x.rstrip(':').strip().lower() for k in kept.values() for x in re.findall(r'\*\*([^*]+)\*\*', k)}
        for term in terms:
            if term.lower() in keptbold: continue
            pat=r'(?<![\w-])'+re.escape(term.strip('`'))+r'(s|es)?(?![\w-])'
            for n,k in kept.items():
                if only and n not in only: continue
                for mm in re.finditer(pat, k, re.I):
                    line=k[:mm.start()].count('\n')+1
                    print(f'dangling [{f}] {n}:{line} "{term}"')
    print('OK' if ok else 'FAIL')
if __name__=='__main__':
    args=[a for a in sys.argv[1:] if a!='--no-orig']
    main(set(args), '--no-orig' in sys.argv)
