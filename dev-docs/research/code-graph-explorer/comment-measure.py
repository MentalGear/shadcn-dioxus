import importlib.util, re, subprocess, sys, os, json, collections
R="/home/user/shadcn-dioxus"
spec=importlib.util.spec_from_file_location("cs",R+"/scripts/comment-scan.py"); cs=importlib.util.module_from_spec(spec); spec.loader.exec_module(cs)
files=[f for f in subprocess.check_output(["git","-C",R,"ls-files","primitives/src"],text=True).split() if f.endswith(".rs")]
ITEM=re.compile(r"^\s*pub\s+(?:async\s+|unsafe\s+|const\s+)*(fn|struct|enum|trait|const|static|type|mod)\s+\w+")
FIELD=re.compile(r"^\s*pub\s+\w+\s*:")
TODO=re.compile(r"\b(TODO|FIXME|HACK|XXX)\b")
rows={}
for f in files:
    t=open(R+"/"+f,encoding="utf-8",errors="replace").read()
    lines=t.split("\n"); code=cs.code_of(t,keep_strings=True).split("\n")
    cl=set(); doc=0; tok=0; todo=0; hist=0; docl=set(); js=0
    for kind,i,j,p in cs._walk_rust(t):
        if kind=="comment":
            isdoc=t.startswith("///",i) or t.startswith("//!",i)
            for ln,body in p:
                cl.add(ln); tok+=len(body.split())
                if isdoc: docl.add(ln)
                if TODO.search(body): todo+=1
                if cs.history_reason(body): hist+=1
        elif len(p[3])>8:
            for ln,body in cs._embedded_comments(p[3],p[4]): js+=1
    codelines=sum(1 for k,c in enumerate(code,1) if c.strip())
    pubs=0;doced=0;fp=0;fd=0
    for k,l in enumerate(lines,1):
        m=ITEM.match(l); fm=FIELD.match(l) and not l.strip().startswith("pub fn")
        if not (m or fm): continue
        if "pub(" in l.split("{")[0].split(":")[0][:12]: continue
        q=k-1; has=False
        while q>=1:
            s=lines[q-1].strip()
            if s.startswith("///"): has=True;break
            if s.startswith("#[") or s.startswith("//") or s=="" and False: q-=1; continue
            if s.startswith(")]") or s.endswith(",") and s.startswith("#"): q-=1; continue
            break
        if m: pubs+=1; doced+=has
        else: fp+=1; fd+=has
    rows[f]=dict(code=codelines,com=len(cl),doc=len(docl),tok=tok,todo=todo,hist=hist,js=js,pubs=pubs,doced=doced,fields=fp,fdoced=fd)
json.dump(rows,open("measured.json","w"),indent=0)
def agg(sel):
    a=collections.Counter()
    for f in sel:
        for k,v in rows[f].items(): a[k]+=v
    return a
def line(n,a):
    print(f"{n:28s} files={len(a['_n']) if '_n' in a else ''} code={a['code']:6d} com={a['com']:6d} ratio={a['com']/max(1,a['code']):.2f} docl={a['doc']:5d} pub={a['pubs']:4d} cov={a['doced']/max(1,a['pubs'])*100:5.1f}% fld={a['fields']:4d} fcov={a['fdoced']/max(1,a['fields'])*100:5.1f}% todo={a['todo']} hist={a['hist']} js={a['js']}")
top=collections.defaultdict(list)
for f in rows:
    parts=f.split("/"); top[parts[2].replace(".rs","") if len(parts)>2 else f].append(f)
line("TOTAL",agg(rows))
for m,fs in sorted(top.items(),key=lambda x:-agg(x[1])['code'])[:14]: line(m,agg(fs))
big=[f for f in rows if rows[f]['code']>=100]
rk=sorted(big,key=lambda f:rows[f]['com']/rows[f]['code'])
print("sparsest(>=100 code)");[print(f,rows[f]['code'],rows[f]['com'],round(rows[f]['com']/rows[f]['code'],2)) for f in rk[:10]]
print("densest");[print(f,rows[f]['code'],rows[f]['com'],round(rows[f]['com']/rows[f]['code'],2)) for f in rk[::-1][:10]]
print(len(rows),"files")
