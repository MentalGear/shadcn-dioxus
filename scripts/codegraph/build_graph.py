# SCIP index + static scans -> graph.json (file-level edges; the page rolls up to component/crate).
# usage: python3 -I build_graph.py <index.scip> <graph.json>   (stdlib only; the repo root is two levels above this file)
# Model: every node is a file {id, comp, items}; `items` is its definitions count. Edges are [src, dst, weight, kind] over
# node indexes. The comment metrics are not here: they depend on file text only and live in comments.json
# (build_comments.py), which the page joins to the nodes by file id.
import sys, os, re, json, collections
REPO=os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
scip, out = sys.argv[1], sys.argv[2]
def varint(b,i):
    r=s=0
    while True:
        x=b[i];i+=1;r|=(x&0x7f)<<s;s+=7
        if x<0x80:return r,i
def fields(b):
    i=0;n=len(b)
    while i<n:
        k,i=varint(b,i);f,w=k>>3,k&7
        if w==0:v,i=varint(b,i)
        elif w==2:
            l,i=varint(b,i);v=b[i:i+l];i+=l
        elif w==1:v=b[i:i+8];i+=8
        elif w==5:v=b[i:i+4];i+=4
        else:raise Exception(w)
        yield f,w,v
data=open(scip,'rb').read()
docs=[]
for f,w,v in fields(data):
    if f==2:
        d={'path':None,'occ':[]}
        for ff,_,vv in fields(v):
            if ff==1:d['path']=vv.decode()
            elif ff==2:
                sym=None;roles=0
                for f3,_,v3 in fields(vv):
                    if f3==2:sym=v3.decode()
                    elif f3==3:roles=v3
                d['occ'].append((sym,roles))
        docs.append(d)
WS={'shadcn-dioxus','dioxus-attributes','preview','dioxus-primitives','test-harness','attr-spread-collision-checker'}
def pkg(sym):
    p=sym.split(' ',4); return p[2] if len(p)>3 else None
defs={};items=collections.Counter()
for d in docs:
    for sym,r in d['occ']:
        if r&1 and not sym.startswith('local '):
            defs[sym]=d['path'];items[d['path']]+=1
STD={'core','alloc','std'}
# Symbols with no definition occurrence of their own. rust-analyzer emits none for what a macro generates (derive output,
# #[component] and Props builders) and none for a module, so each is mapped to the definition that owns it:
#   owner   the type or function named in the descriptor (`impl#[T<..>][Trait]m().`, `T#0.`, `Props...Builder`,
#           `...WithOwner`, `F_completions/Component#F#` -> `F().`) when that has a definition
#   module  else the file of the module the symbol sits in (`a/b/` -> a/b.rs or a/b/mod.rs), shortening the path until a file exists
def split_desc(desc):
    parts=[];cur='';depth=0;bt=False
    for ch in desc:
        if ch=='`':bt=not bt
        elif not bt:
            if ch=='[':depth+=1
            elif ch==']':depth-=1
        if ch=='/' and not bt and depth==0:parts.append(cur);cur=''
        else:cur+=ch
    return parts,cur
def crate_src(path):
    d=os.path.dirname(path)
    while d and not os.path.isfile(os.path.join(REPO,d,'Cargo.toml')):d=os.path.dirname(d)
    return os.path.join(d,'src') if d else None
pkg_src={}
for sym,path in defs.items():
    pkg_src.setdefault(pkg(sym),crate_src(path))
def head(sym):return ' '.join(sym.split(' ',4)[:4])+' '
extra_files=set()
def module_file(sym):
    src=pkg_src.get(pkg(sym))
    if not src:return None
    mods,_=split_desc(sym.split(' ',4)[4])
    while True:
        m=[x.strip('`').removeprefix('r#') for x in mods if x]
        base=os.path.join(src,*m) if m else None
        for c in ((base+'.rs',os.path.join(base,'mod.rs')) if m else (os.path.join(src,'lib.rs'),os.path.join(src,'main.rs'))):
            if os.path.isfile(os.path.join(REPO,c)):return c
        if not m:return None
        mods=mods[:-1]
def owner_names(last):
    names=[]
    for t in re.findall(r'\[`?([A-Za-z_]\w*)',last):names.append(t)
    m=re.match(r'([A-Za-z_]\w*)#',last)
    if m:names.append(m.group(1))
    return names
# a type whose own definition occurrence is missing still has its fields and variants (`ButtonProps#variant.`)
owned={}
for sym,path in defs.items():
    h=head(sym);desc=sym.split(' ',4)[4];i=desc.find('#')
    if i>0 and not desc.startswith('impl#') and '/impl#' not in desc[:i+1]:owned.setdefault(h+desc[:i+1],path)
resolved={}
def resolve(sym):
    if sym in resolved:return resolved[sym]
    h=head(sym);mods,last=split_desc(sym.split(' ',4)[4])
    pre=''.join(x+'/' for x in mods)
    res=None
    if mods and mods[-1].endswith('_completions'):
        n=mods[-1][:-len('_completions')];pre=''.join(x+'/' for x in mods[:-1])
        for c in (pre+n+'().',pre+n+'#'):
            if h+c in defs:res=(defs[h+c],'owner');break
    elif last:
        for n in owner_names(last):
            cands=[n]
            for suf in ('Builder','WithOwner'):
                if n.endswith(suf):cands.append(n[:-len(suf)])
            for c in cands:
                for k in ('#','().'):
                    t=defs.get(h+pre+c+k) or (owned.get(h+pre+c+k) if k=='#' else None)
                    if t:res=(t,'owner');break
                if res:break
            if res:break
    if not res:
        f=module_file(sym)
        if f:res=(f,'module')
    resolved[sym]=res
    return res
rtier=collections.Counter()
fe=collections.Counter();unres=0;ext=collections.Counter();stdn=0;itemedges=set()
for d in docs:
    for sym,r in d['occ']:
        if r&1 or sym.startswith('local '):continue
        t=defs.get(sym)
        if t:
            if t!=d['path']:fe[(d['path'],t,'rs')]+=1;itemedges.add((d['path'],sym))
        else:
            p=pkg(sym)
            if p in STD:stdn+=1
            elif p in WS or p is None:
                q=resolve(sym)
                if q:
                    rtier[q[1]]+=1;extra_files.add(q[0])
                    if q[0]!=d['path']:fe[(d['path'],q[0],'rs')]+=1;itemedges.add((d['path'],sym))
                else:unres+=1
            else:ext[(d['path'],'ext:'+p)]+=1
for (a,b),c in ext.items():fe[(a,b,'ext')]+=c
files={d['path'] for d in docs}|extra_files
# static scans: asset!/include_str! refs, TS imports
asset_re=re.compile(r'(?:include_str|include_bytes|asset)!\(\s*"([^"]+)"')
def crate_dir(p):return p.split('/')[0]
for d in docs:
    p=d['path'];txt=open(os.path.join(REPO,p),errors='replace').read()
    for m in asset_re.finditer(txt):
        q=m.group(1)
        cands=[q.lstrip('/'),os.path.join(crate_dir(p),q.lstrip('/')),os.path.normpath(os.path.join(os.path.dirname(p),q))]
        for c in cands:
            if os.path.isfile(os.path.join(REPO,c)):
                files.add(c);fe[(p,c,'asset')]+=1;break
ts_files=[]
for base in ('primitives/src/ts','playwright','primitives/ts-test'):
    for root,_,fs in os.walk(os.path.join(REPO,base)):
        if 'node_modules' in root:continue
        for f in fs:
            if f.endswith('.ts'):ts_files.append(os.path.relpath(os.path.join(root,f),REPO))
imp=re.compile(r'''(?:import|export)\s[^;'"]*?from\s*['"](\.[^'"]+)['"]|import\s*['"](\.[^'"]+)['"]''')
tsset=set(ts_files)
for p in ts_files:
    files.add(p)
    for m in imp.finditer(open(os.path.join(REPO,p),errors='replace').read()):
        q=os.path.normpath(os.path.join(os.path.dirname(p),m.group(1) or m.group(2)))
        for c in (q,q+'.ts',q+'/index.ts',re.sub(r'\.js$','.ts',q)):
            if c in tsset:fe[(p,c,'ts')]+=1;break
# themed components: component.json marks one (it depends on dioxus-primitives)
cdirs={}
for d in sorted(os.listdir(os.path.join(REPO,'preview/src/components'))):
    cj=os.path.join(REPO,'preview/src/components',d,'component.json')
    if not os.path.isfile(cj):continue
    deps=[x.get('name') for x in json.load(open(cj)).get('cargoDependencies',[])]
    if 'dioxus-primitives' in deps:cdirs[d]=os.path.dirname(cj)
# a themed component's `mod.rs` holds no definitions, so the index has no document for it; it is a node with an edge to
# each module it declares (`mod component;`)
mod_re=re.compile(r'^\s*(?:pub\s+)?mod\s+(\w+)\s*;',re.M)
for d in cdirs:
    p='preview/src/components/%s/mod.rs'%d
    if os.path.isfile(os.path.join(REPO,p)):
        files.add(p)
        for m in mod_re.finditer(open(os.path.join(REPO,p),errors='replace').read()):
            q='preview/src/components/%s/%s.rs'%(d,m.group(1))
            if os.path.isfile(os.path.join(REPO,q)):files.add(q);fe[(p,q,'rs')]+=1
# component mapping
def comp(path):
    if path.startswith('ext:'):return path,'external',path[4:]
    parts=path.split('/')
    c=parts[0]
    if c=='preview':
        if parts[1:3]==['src','components'] and len(parts)>4:return 'preview:'+parts[3],'preview',parts[3]
        if len(parts)>2 and parts[1]=='src':
            s=os.path.splitext(parts[2])[0]
            return 'preview:'+s,'preview','·'+s if s!='components' else '·components'
        return 'preview:·'+os.path.splitext(parts[1])[0],'preview','·'+os.path.splitext(parts[1])[0]
    if c=='primitives':
        if parts[1]=='src' and len(parts)>2:
            s=os.path.splitext(parts[2])[0]
            if parts[2]=='ts' or parts[2]=='js':s=parts[2]+'/'+(os.path.splitext(parts[3])[0] if len(parts)>4 else 'root')
            return 'primitives:'+s,'primitives',s
        return 'primitives:·'+parts[1],'primitives','·'+parts[1]
    if c=='playwright':
        return 'playwright:'+('specs' if path.endswith('.spec.ts') else 'support'),'playwright','specs' if path.endswith('.spec.ts') else 'support'
    s=parts[1] if len(parts)>1 else '·root'
    return c+':'+os.path.splitext(s)[0],c,os.path.splitext(s)[0]
comps={};nodes=[]
allf=sorted(files|{b for (_,b,k) in fe if k=='ext'})
fidx={f:i for i,f in enumerate(allf)}
for f in allf:
    cid,cr,name=comp(f)
    comps.setdefault(cid,{'id':cid,'crate':cr,'name':name})
# Pairing: a themed preview component and the primitive it wraps. component.json marks a themed component (it depends on
# dioxus-primitives); the primitive is read from the `dioxus_primitives::<module>` paths in that component's own sources.
# A module named like the component wins. Otherwise the module referenced most often wins, if it is unique and is not a
# helper (a module with no preview component of its own that three or more components use, or any module ten or more
# components use, such as `direction`). Each side lists the other in `pair`; `paired` is true when `pair` is not empty.
refs={}
use_re=re.compile(r'dioxus_primitives::(\w+)')
brace_re=re.compile(r'dioxus_primitives::\{([^}]*)\}')
for d,dd in cdirs.items():
    cnt=collections.Counter()
    for root,_,fs in os.walk(dd):
        for f in fs:
            if not f.endswith('.rs'):continue
            t=open(os.path.join(root,f),errors='replace').read()
            cnt.update(m.group(1) for m in use_re.finditer(t))
            for m in brace_re.finditer(t):cnt.update(re.findall(r'(?:^|,)\s*(\w+)',m.group(1)))
    refs[d]=cnt
prim={c['name']:c['id'] for c in comps.values() if c['crate']=='primitives' and not c['name'].startswith('·') and '/' not in c['name']}
referrers=collections.Counter(m for d,c in refs.items() for m in c if m!=d)
def helper(m):return referrers[m]>=10 or (m not in cdirs and referrers[m]>=3)
pairs=[]
for d,cnt in sorted(refs.items()):
    if d in prim:pairs.append((d,d));continue
    c=[(n,m) for m,n in cnt.items() if m in prim and not helper(m)]
    if c:
        top=max(n for n,_ in c);best=[m for n,m in c if n==top]
        if len(best)==1:pairs.append((d,best[0]))
for c in comps.values():c['pair']=[]
for d,m in pairs:
    a,b='preview:'+d,prim[m]
    if a in comps and b in comps:
        comps[a]['pair'].append(b);comps[b]['pair'].append(a)
for c in comps.values():
    c['pair']=sorted(set(c['pair']));c['paired']=bool(c['pair'])
# playwright specs -> component by name (select.spec.ts, native-select.spec.ts -> native_select)
byname={c['name']:c for c in comps.values() if c['crate']=='preview'}
for f in list(allf):
    m=re.match(r'playwright/([a-z0-9_-]+)\.spec\.ts$',f)
    if m:
        n=m.group(1).replace('-','_')
        cand=[k for k in byname if n==k or n.startswith(k+'_')]
        if cand:
            k=max(cand,key=len)
            for leaf in ('mod.rs','component.rs'):
                t='preview/src/components/%s/%s'%(k,leaf)
                if t in fidx:fe[(f,t,'test')]+=1;break
nodes=[]
for f in allf:
    cid,cr,name=comp(f)
    nodes.append({'id':f,'comp':cid,'items':items.get(f,0)})
edges=[[fidx[a],fidx[b],c,k] for (a,b,k),c in fe.items() if a in fidx and b in fidx]
g={'meta':{'resolved_by_owner':rtier['owner'],'resolved_by_module':rtier['module'],'scip_documents':len(docs),'items_defined':len(defs),'unresolved_workspace_refs':unres,'std_refs':stdn,'item_level_edges':len(itemedges)},
   'comps':list(comps.values()),'nodes':nodes,'edges':edges}
json.dump(g,open(out,'w'),separators=(',',':'))
kinds=collections.Counter(e[3] for e in edges)
print('files',len(nodes),'comps',len(comps),'edges',len(edges),dict(kinds),g['meta'])
