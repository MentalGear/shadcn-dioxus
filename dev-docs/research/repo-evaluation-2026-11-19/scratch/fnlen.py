import os,re,sys
res=[]
fnre=re.compile(r'^\s*(pub(\([^)]*\))?\s+)?(async\s+)?(const\s+)?fn\s+(\w+)')
for root in sys.argv[1:]:
  for dp,dn,fn in os.walk(root):
    if 'target' in dp: continue
    for f in fn:
      if not f.endswith('.rs') or 'test' in f: continue
      p=os.path.join(dp,f); L=open(p).read().split('\n')
      i=0
      while i<len(L):
        m=fnre.match(L[i])
        if m:
          depth=0; started=False; j=i
          while j<len(L):
            s=L[j]
            depth+=s.count('{')-s.count('}')
            if '{' in s: started=True
            if started and depth<=0: break
            if not started and s.rstrip().endswith(';'): break
            j+=1
          # params count approx
          res.append((j-i+1,p,i+1,m.group(5)))
          i+=1
        else: i+=1
res.sort(reverse=True)
for r in res[:25]: print(*r)
