import collections,re,subprocess,sys,json
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
        yield f,w,v
def packed(b):
    i=0;o=[]
    while i<len(b):
        v,i=varint(b,i);o.append(v)
    return o
data=open('target-codegraph/index.scip','rb').read()
occ=collections.defaultdict(list)  # path -> (line,col,sym,roles)
for f,w,v in fields(data):
    if f!=2: continue
    path=None
    for ff,_,vv in fields(v):
        if ff==1:path=vv.decode()
        elif ff==2:
            rng=sym=None;roles=0
            for f3,w3,v3 in fields(vv):
                if f3==1:rng=packed(v3)
                elif f3==2:sym=v3.decode()
                elif f3==3:roles=v3
            occ[path].append((rng[0]+1,rng[1],sym,roles))
import pickle;pickle.dump(dict(occ),open(sys.argv[1],'wb'))
