import pickle,collections,subprocess
occ=pickle.load(open('/tmp/claude-0/-home-user-shadcn-dioxus/2f9af3b0-3995-5f94-a96b-6c0dfde7973a/scratchpad/verify-ac/occ.pkl','rb'))
pat=[('format!','macros/format!'),('to_string','[ToString]to_string().'),('vec!','macros/vec!'),('Iterator::collect','Iterator#collect().'),('String::clone','[String][Clone]clone().'),('Vec::clone','`Vec<T, A>`][Clone]clone().'),('Option::clone','`Option<T>`][Clone]clone().'),('Rc::clone','`Rc<T, A>`][Clone]clone().'),('to_owned','ToOwned]to_owned().'),('to_vec','to_vec().')]
c=collections.Counter()
for p,l in occ.items():
    for ln,col,s,r in l:
        if 'rust/library' not in s: continue
        for k,v in pat:
            if s.endswith(v) or (k in('format!','vec!') and s.endswith(v)): c[k]+=1
print(dict(c))
# format! occurrences: text at (line,col) starts with 'format'
bad=n=0
for p,l in occ.items():
    if not p.endswith('.rs'):continue
    lines=open(p,errors='replace').read().split('\n')
    for ln,col,s,r in l:
        if 'rust/library' in s and s.endswith('macros/format!'):
            n+=1; bad+= not lines[ln-1][col:].startswith('format')
print('format! occ',n,'text-at-position mismatches',bad)
g=subprocess.run("git ls-files '*.rs' | xargs grep -o 'format!(' | wc -l",shell=True,capture_output=True,text=True).stdout.strip()
print('grep format!( over all tracked .rs',g)
tot=0
for p in subprocess.run("git ls-files '*.rs'",shell=True,capture_output=True,text=True).stdout.split(): tot+=open(p,errors='replace').read().count('.clone()')
print('grep .clone() all tracked rs',tot)
for p in ['primitives/src/questionnaire.rs','primitives/src/bridge/session.rs','primitives/src/questionnaire/model.rs','primitives/src/avatar.rs','primitives/src/tag_group.rs']:
    print(p,'grep .clone()',open(p).read().count('.clone()'),'scip clone() occ any package',sum(1 for ln,col,s,r in occ[p] if 'clone()' in s),'grep lines',sum(1 for l in open(p) if '.clone()' in l))
p='primitives/src/chart/components/chart.rs'
print([(ln,col,s.split(' ',4)[4]) for ln,col,s,r in sorted(occ[p]) if 700<=ln<=708 and 'Clone' in s])
