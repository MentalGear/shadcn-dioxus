# minimal SCIP protobuf reader -> per-document stats + file->file edges
import sys, collections, json
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
data=open(sys.argv[1],'rb').read()
docs=[];ext=0
for f,w,v in fields(data):
    if f==1:
        for ff,_,vv in fields(v):
            if ff==2:print('tool',[x[2] for x in fields(vv)][:2])
    elif f==2:
        d={'path':None,'occ':[],'syms':0}
        for ff,_,vv in fields(v):
            if ff==1:d['path']=vv.decode()
            elif ff==2:
                o={'sym':None,'roles':0}
                for f3,_,v3 in fields(vv):
                    if f3==2:o['sym']=v3.decode()
                    elif f3==3:o['roles']=v3
                d['occ'].append(o)
            elif ff==3:d['syms']+=1
        docs.append(d)
    elif f==3:ext+=1
print('documents',len(docs),'occurrences',sum(len(d['occ']) for d in docs),'symbolinfos',sum(d['syms'] for d in docs),'external_symbols',ext)
# definition map: symbol -> file (role bit 1 = Definition)
defs={}
for d in docs:
    for o in d['occ']:
        if o['roles']&1 and not o['sym'].startswith('local '):defs[o['sym']]=d['path']
edges=collections.Counter()
for d in docs:
    for o in d['occ']:
        if o['roles']&1 or o['sym'].startswith('local '):continue
        t=defs.get(o['sym'])
        if t and t!=d['path']:edges[(d['path'],t)]+=1
print('file nodes',len(docs),'file->file edges (resolved in-workspace refs)',len(edges))
for (a,b),c in edges.most_common(8):print(c,a,'->',b)
ex=[o['sym'] for d in docs[:3] for o in d['occ']][:0]
import itertools
print('sample symbols:')
for s in itertools.islice((k for k in defs),6):print(' ',s)
json.dump([[a,b,c] for (a,b),c in edges.items()],open(sys.argv[2],'w'))
