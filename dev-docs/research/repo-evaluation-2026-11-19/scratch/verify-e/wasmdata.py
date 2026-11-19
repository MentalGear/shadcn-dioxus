import sys,re,zlib
b=open(sys.argv[1],'rb').read()
def leb(p):
    r=0;s=0
    while True:
        x=b[p];p+=1;r|=(x&0x7f)<<s;s+=7
        if not x&0x80:return r,p
assert b[:4]==b'\0asm'
p=8;secs=[]
while p<len(b):
    sid=b[p];p+=1;ln,p=leb(p);secs.append((sid,p,ln));p+=ln
names={0:'custom',1:'type',2:'import',3:'function',4:'table',5:'memory',6:'global',7:'export',8:'start',9:'element',10:'code',11:'data',12:'datacount'}
for sid,st,ln in secs: print('section',names.get(sid,sid),ln, 'gz',len(zlib.compress(b[st:st+ln],9)) if ln>100000 else '')
# data section
sid,st,ln=[s for s in secs if s[0]==11][0]
p=st;n,p=leb(p);segs=[]
for i in range(n):
    flag,p=leb(p)
    if flag==0:
        # const expr until 0x0b
        while b[p]!=0x0b:p+=1
        p+=1
    elif flag==2:
        _,p=leb(p)
        while b[p]!=0x0b:p+=1
        p+=1
    sz,p=leb(p);segs.append(b[p:p+sz]);p+=sz
print('segments',n,'total data bytes',sum(map(len,segs)))
# classify: split every segment into maximal runs of printable ASCII/UTF-8 text (>=40 bytes), then classify by content
runre=re.compile(rb'(?:[\x20-\x7e\n\r\t]|[\xc2-\xf4][\x80-\xbf]+){40,}')
rust=css=html=other=0; nonrun=0
rustb=bytearray();cssb=bytearray()
def cls(t):
    s=t.decode('utf-8','replace')
    # CSS: many "{" "}" ":" ";" and no "fn "/"use "/"let "
    rs=len(re.findall(r'\b(fn|use|let|pub|impl|struct|mod|rsx!|mut|match|self|Some|None)\b',s))
    cs=len(re.findall(r'[a-z\-]+\s*:\s*[^;{}]+;',s))+s.count('{')
    hs=len(re.findall(r'</?[a-z][a-z0-9\-]*[ >/]',s))
    m=max(rs,cs,hs)
    if m*1000 < len(s)*2: return 'other'   # density threshold: <2 hits per 1000 bytes
    return {rs:'rust',cs:'css',hs:'html'}[m]
tot={'rust':0,'css':0,'html':0,'other':0}
for sg in segs:
    for m in runre.finditer(sg):
        t=m.group(0);c=cls(t);tot[c]+=len(t)
        if c=='rust':rustb+=t+b'\n'
        if c=='css':cssb+=t+b'\n'
covered=sum(tot.values())
print('text runs>=40 total',covered,'of',sum(map(len,segs)),'non-text',sum(map(len,segs))-covered)
for k,v in tot.items():print(k,v)
print('rust gz',len(zlib.compress(bytes(rustb),9)),'css gz',len(zlib.compress(bytes(cssb),9)))
open(sys.argv[2]+'/rust.txt','wb').write(rustb);open(sys.argv[2]+'/css.txt','wb').write(cssb)
