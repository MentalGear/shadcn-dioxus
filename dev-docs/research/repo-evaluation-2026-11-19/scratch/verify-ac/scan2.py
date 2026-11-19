# independent Rust scanner: real tokenizer, token-level fn detection, file-level cfg(test) mod exclusion
import os,re,sys,subprocess,collections,json
R='/home/user/shadcn-dioxus'
def lex(s):
    toks=[];i=0;n=len(s);line=1
    def adv(a,b):
        nonlocal line
        line+=s.count('\n',a,b)
    while i<n:
        c=s[i]
        if c=='\n': line+=1;i+=1;continue
        if c.isspace(): i+=1;continue
        if s.startswith('//',i):
            j=s.find('\n',i);i=n if j<0 else j;continue
        if s.startswith('/*',i):
            d=1;j=i+2
            while j<n and d:
                if s.startswith('/*',j):d+=1;j+=2
                elif s.startswith('*/',j):d-=1;j+=2
                else:j+=1
            adv(i,j);i=j;continue
        m=re.compile(r'b?r(#*)"').match(s,i)
        if m:
            end='"'+m.group(1);j=s.find(end,m.end());j=n if j<0 else j+len(end)
            toks.append(('str','',line));adv(i,j);i=j;continue
        if c=='"' or (c=='b' and s.startswith('b"',i)):
            j=i+(1 if c=='"' else 2)
            while j<n and s[j]!='"': j+=2 if s[j]=='\\' else 1
            toks.append(('str','',line));adv(i,j+1);i=j+1;continue
        if c=="'" or s.startswith("b'",i):
            k=i+(1 if c=="'" else 2)
            m=re.compile(r"(\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}|.)|[^\\'\n])'").match(s,k)
            if m: toks.append(('chr','',line));i=m.end();continue
            m=re.compile(r"'[A-Za-z_]\w*").match(s,i)
            if m: toks.append(('life',m.group(),line));i=m.end();continue
        m=re.compile(r'[A-Za-z_]\w*').match(s,i)
        if m: toks.append(('id',m.group(),line));i=m.end();continue
        m=re.compile(r'\d[\w.]*').match(s,i)
        if m: toks.append(('num',m.group(),line));i=m.end();continue
        toks.append(('p',c,line));i+=1
    return toks
def match_close(t,i):
    o=t[i][1];c={'{':'}','(':')','[':']'}[o];d=0
    for j in range(i,len(t)):
        if t[j][0]=='p':
            if t[j][1]==o:d+=1
            elif t[j][1]==c:
                d-=1
                if d==0:return j
    return len(t)-1
def analyze(path):
    s=open(path,errors='replace').read();t=lex(s);n=len(t)
    skip=[] # token ranges that are test
    testmods=[]
    i=0
    # find #[cfg(test)] attrs (also cfg(all(test,...)) -> only plain cfg(test))
    for i in range(n-3):
        if t[i][1]=='#' and t[i+1][1]=='[':
            e0=match_close(t,i+1)
            body=t[i+2:e0]
            names=[x[1] for x in body if x[0]=='id']
            istest=False
            if names and names[0]=='cfg' and 'test' in names and 'not' not in names: istest=True
            if names and names[-1] in('test','wasm_bindgen_test') and (len(names)==1 or names[0] in('tokio','wasm_bindgen_test')): istest=True
            if not istest: continue
            j=e0+1
            while j<n and t[j][1]=='#' and t[j+1][1]=='[': j=match_close(t,j+1)+1
            k=j;d=0
            while k<n:
                if t[k][0]=='p':
                    if t[k][1] in '([':d+=1
                    elif t[k][1] in ')]':d-=1
                    elif d==0 and t[k][1]=='{':break
                    elif d==0 and t[k][1]==';':break
                k+=1
            if k<n and t[k][1]=='{': skip.append((i,match_close(t,k)))
            elif k<n:
                if any(x[1]=='mod' for x in t[j:min(j+5,k+1)]):
                    nm=[x[1] for x in t[j:k] if x[0]=='id' and x[1] not in('pub','mod','crate','super','in','self')]
                    if nm: testmods.append(nm[-1])
                skip.append((i,k))
    return s,t,skip,testmods
def in_skip(skip,i):
    for a,b in skip:
        if a<=i<=b:return True
    return False
files=subprocess.run("git ls-files '*.rs'",shell=True,capture_output=True,text=True,cwd=R).stdout.split()
# file-level test modules
testfiles=set()
info={}
for f in files:
    s,t,skip,tm=analyze(os.path.join(R,f));info[f]=(s,t,skip)
    d=os.path.dirname(f);b=os.path.basename(f)[:-3]
    base=d if b in('lib','main','mod') else os.path.join(d,b)
    for m in tm:
        for cand in (os.path.join(base,m+'.rs'),os.path.join(base,m,'mod.rs')): testfiles.add(cand)
def istestpath(f): return '/tests/' in f or f.endswith('/tests.rs') or f in testfiles
res=collections.defaultdict(lambda:collections.defaultdict(list))
ASS={'assert','assert_eq','assert_ne','debug_assert','debug_assert_eq','debug_assert_ne'}
cnt=collections.defaultdict(collections.Counter)
for f in files:
    s,t,skip=info[f];crate=f.split('/')[0] if f.split('/')[0] not in('scripts','playwright') else f.split('/')[0]+':'+f.split('/')[1]
    if istestpath(f): continue
    n=len(t)
    for i in range(n):
        if in_skip(skip,i):continue
        tk=t[i]
        if tk[0]=='id' and tk[1]=='fn' and i+1<n and t[i+1][0]=='id':
            # find body
            k=i+2;d=0
            while k<n:
                if t[k][0]=='p':
                    if t[k][1] in '([':d+=1
                    elif t[k][1] in ')]':d-=1
                    elif d==0 and t[k][1] in '{;':break
                k+=1
            if k>=n or t[k][1]==';':continue
            e=match_close(t,k)
            # return type Element?
            sig=t[i:k]
            ret=''
            for x in range(len(sig)-1):
                if sig[x][1]=='-' and sig[x+1][1]=='>': ret=' '.join(y[1] for y in sig[x+2:])
            # attrs: walk back over attributes
            comp=False;j=i-1
            while j>=0 and t[j][1] in('pub','async','const','unsafe','crate','in','super','self') or (j>=0 and t[j][1]==')' ): j-=1
            if j>=0 and t[j][1]==']':
                # attribute block(s) preceding
                while j>=0 and t[j][1]==']':
                    d=0;q=j
                    while q>=0:
                        if t[q][1]==']':d+=1
                        elif t[q][1]=='[':
                            d-=1
                            if d==0:break
                        q-=1
                    if any(y[1]=='component' for y in t[q:j]): comp=True
                    j=q-2  # skip '#'
                    while j>=0 and t[j][1] in('pub','async','const','unsafe','crate','in','super','self',')'): j-=1
            if 'Element' in ret.split():comp=True
            L=t[e][2]-t[i][2]+1
            na=sum(1 for q in range(k,e) if t[q][0]=='id' and t[q][1] in ASS and t[q+1][1]=='!')
            q2=i-1
            while q2>=0 and t[q2][1] in('async','const','unsafe','extern') : q2-=1
            if q2>=0 and t[q2][0]=='str': q2-=1
            while q2>=0 and t[q2][1] in('async','const','unsafe','extern') : q2-=1
            if q2>=0 and t[q2][1]==')':
                while q2>=0 and t[q2][1]!='(' : q2-=1
                q2-=1
            ispub=q2>=0 and t[q2][1]=='pub'
            # stricter pub: token immediately before fn qualifiers is pub or pub(...)
            res[crate]['comp' if comp else 'non'].append((L,f,t[i][2],t[i+1][1],na,ispub))
    for i in range(len(t)):
        if in_skip(skip,i):continue
        if t[i][0]=='id' and t[i][1] in ASS|{'panic','unreachable'} and i+1<len(t) and t[i+1][1]=='!': cnt[crate][t[i][1]]+=1
        if t[i][0]=='id' and t[i][1] in('expect','unwrap') and i>0 and t[i-1][1]=='.' and t[i+1][1]=='(' : cnt[crate][t[i][1]]+=1
out=[]
for c in sorted(res):
    for kind in('comp','non'):
        v=[x[0] for x in res[c][kind]]
        if v: out.append(f"{c:45s} {kind:5s} n={len(v):5d} max={max(v):4d} >70={sum(1 for x in v if x>70):4d} >100={sum(1 for x in v if x>100)}")
print('\n'.join(out))
tot=sum(1 for c in res for x in res[c]['non'] if x[0]>70);print('non-component >70 all:',tot)
print('non-comp >70 by crate',{c:sum(1 for x in res[c]['non'] if x[0]>70) for c in res})
print('testfiles excluded',sorted(testfiles & set(files)))
print('assert-family/panic/unwrap/expect',{c:dict(v) for c,v in cnt.items()})
nf={c:len(res[c]['comp'])+len(res[c]['non']) for c in res};print('fns',nf)
json.dump({c:{k:v for k,v in d.items()} for c,d in res.items()},open('/tmp/claude-0/-home-user-shadcn-dioxus/2f9af3b0-3995-5f94-a96b-6c0dfde7973a/scratchpad/verify-ac/scan2.json','w'))
if len(sys.argv)>1:
    c=collections.Counter()
    for f in files:
        s,t,skip=info[f]
        if istestpath(f) or not f.startswith('primitives/'):continue
        for i in range(len(t)-1):
            if in_skip(skip,i):continue
            if t[i][0]=='id' and t[i][1] in ASS and t[i+1][1]=='!': c[f]+=1
    print(c.most_common(12))

for c in('primitives','decision-kernel','cli'):
    fs=[x for x in res[c]['non']+(res[c]['comp'] if False else []) if x[5]]
    print(c,'pub non-component fns',len(fs),'with assert-family',sum(1 for x in fs if x[4]>0))
