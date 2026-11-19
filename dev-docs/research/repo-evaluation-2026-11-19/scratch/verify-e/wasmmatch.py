import sys,os,glob,re,zlib
exec(open(sys.argv[1]).read().split("# classify")[0].replace("print(","(lambda *a,**k:None)(") ) if False else None
b=open(sys.argv[1],'rb').read()
def leb(p):
    r=0;s=0
    while True:
        x=b[p];p+=1;r|=(x&0x7f)<<s;s+=7
        if not x&0x80:return r,p
p=8;ds=None
while p<len(b):
    sid=b[p];p+=1;ln,p=leb(p)
    if sid==11:ds=(p,ln)
    p+=ln
p,_=ds;n,p=leb(p);segs=[]
for i in range(n):
    flag,p=leb(p)
    if flag==0:
        while b[p]!=0x0b:p+=1
        p+=1
    sz,p=leb(p);segs.append(b[p:p+sz]);p+=sz
data=b'\0'.join(segs)
root='/home/user/shadcn-dioxus/preview/src/components'
def scan(files,label):
    hit=miss=0;hb=mb=0;seen=set();gz=bytearray()
    for f in files:
        t=open(f,'rb').read()
        if not t.strip():continue
        if t in data: hit+=1;hb+=len(t);gz+=t
        else: miss+=1;mb+=len(t)
    print(label,'files embedded verbatim:',hit,'bytes',hb,'gzip(concat)',len(zlib.compress(bytes(gz),9)),'| not found verbatim:',miss,'bytes',mb)
rs=sorted(glob.glob(root+'/*/component.rs')+glob.glob(root+'/*/variants/*/mod.rs'))
scan(rs,'rust component.rs + variants/*/mod.rs')
css=sorted(glob.glob(root+'/*/style.css')+glob.glob(root+'/*/variants/*/demo.css')+['/home/user/shadcn-dioxus/preview/assets/dx-components-theme.css'])
css=[c for c in css if os.path.exists(c)]
scan(css,'css style.css/demo.css/theme.css')
# also the md/docs
print('--- multiplicity')
tot=0;multi=0
for f in rs:
    t=open(f,'rb').read()
    if not t.strip():continue
    c=data.count(t); tot+=c*len(t); multi+=(c>1)
print('rust: total bytes counting copies',tot,'files with >1 copy',multi)
tot=0
for f in css:
    t=open(f,'rb').read(); c=data.count(t); tot+=c*len(t)
print('css: total bytes counting copies',tot)
# other rust-looking text: count occurrences of marker lines
for m in (b'use dioxus::prelude',b'use super::super::component',b'use dioxus_primitives'):
    print(m.decode(),data.count(m))
# find extra rust: all primitives/src/*.rs, docs_samples
extra=glob.glob('/home/user/shadcn-dioxus/primitives/src/**/*.rs',recursive=True)+glob.glob('/home/user/shadcn-dioxus/preview/src/**/*.rs',recursive=True)
scan([e for e in extra if '/components/' not in e],'other preview/primitives .rs files verbatim')
print('--- residual after removing verbatim files')
rest=data
for f in rs+css:
    t=open(f,'rb').read()
    if t.strip(): rest=rest.replace(t,b'\0')
print('data bytes',len(data),'after removal',len(rest.replace(b'\0',b'')))
runre=re.compile(rb'[\x20-\x7e\n\t]{40,}')
runs=[m.group(0) for m in runre.finditer(rest)]
print('residual printable runs>=40:',len(runs),'bytes',sum(map(len,runs)))
import collections
cl=collections.Counter()
for r in runs:
    s=r[:300].decode('latin1')
    k='html' if re.search(r'<[a-z]+[ >]',s) else 'rustish' if re.search(r'\b(fn|let|pub|use|impl)\b',s) else 'css' if re.search(r'[{};]',s) else 'prose/other'
    cl[k]+=len(r)
print(dict(cl))
big=sorted(runs,key=len,reverse=True)[:3]
for r in big:print(len(r),r[:160])
print('--- OUT_DIR generated files')
O='/home/user/targets/eval-output/wasm32-unknown-unknown/wasm-release/build/preview-7315b5d50bbd1e5c/out'
import subprocess
allf=[os.path.join(d,f) for d,_,fs in os.walk(O) for f in fs]
byext=collections.defaultdict(lambda:[0,0,0,0])
for f in allf:
    t=open(f,'rb').read()
    if not t.strip():continue
    e=os.path.basename(f) if os.path.basename(f) in('notes.html','description.txt','docs_sections.rs') else os.path.splitext(f)[1]
    r=byext[e];r[0]+=1;r[1]+=len(t)
    if t in data:r[2]+=1;r[3]+=len(t)
for e,r in byext.items():print(e,'files',r[0],'bytes',r[1],'embedded verbatim files',r[2],'bytes',r[3])
