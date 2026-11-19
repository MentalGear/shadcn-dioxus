import sys,os,re
pats=sys.argv[2:]
root=sys.argv[1]
for dp,dn,fn in os.walk(root):
    if 'target' in dp or 'tests' in dp.split('/'): continue
    for f in fn:
        if not f.endswith('.rs') or 'test' in f: continue
        p=os.path.join(dp,f); src=open(p).read().split('\n')
        skip=0; depth=0; pending=False
        for i,l in enumerate(src,1):
            s=l.strip()
            if skip:
                depth+=l.count('{')-l.count('}')
                if depth<=0: skip=0
                continue
            if s.startswith('#[cfg(test)]') or s.startswith('#[test]'):
                pending=True; continue
            if pending:
                if s.startswith('#['): continue
                pending=False
                if s.endswith(';'): continue
                depth=l.count('{')-l.count('}')
                if depth>0: skip=1
                continue
            if s.startswith('//'): continue
            for pat in pats:
                if pat in l: print(f"{p}:{i}: {s[:150]}")
