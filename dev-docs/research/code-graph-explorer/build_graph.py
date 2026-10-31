# SCIP index + static scans -> graph.json (file-level edges; client rolls up to component/crate)
import sys, os, re, json, collections
REPO='/home/user/shadcn-dioxus'
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
            elif p in WS or p is None:unres+=1
            else:ext[(d['path'],'ext:'+p)]+=1
for (a,b),c in ext.items():fe[(a,b,'ext')]+=c
files={d['path'] for d in docs}
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
# component names that exist in both preview and primitives => paired
pn=collections.defaultdict(set)
for c in comps.values():
    if c['crate'] in('preview','primitives'):pn[c['name']].add(c['crate'])
for c in comps.values():
    c['paired']=pn.get(c['name'])=={'preview','primitives'}
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
g={'meta':{'scip_documents':len(docs),'items_defined':len(defs),'unresolved_workspace_refs':unres,'std_refs':stdn,'item_level_edges':len(itemedges)},
   'comps':list(comps.values()),'nodes':nodes,'edges':edges}
json.dump(g,open(out,'w'),separators=(',',':'))
kinds=collections.Counter(e[3] for e in edges)
print('files',len(nodes),'comps',len(comps),'edges',len(edges),dict(kinds),g['meta'])
