import re,subprocess,sys
files=subprocess.check_output("git ls-files '*.rs'",shell=True,text=True).split()
tot={"native_pos":0,"native_neg":0,"server":0}
detail=[]
for f in files:
    if not f.startswith(("primitives/","preview/","cli/","decision-kernel/")):continue
    L=open(f).read().split("\n")
    for i,l in enumerate(L):
        m=re.match(r'\s*#\[cfg\((.*)\)\]\s*$',l) or re.match(r'\s*#!\[cfg\((.*)\)\]',l)
        if not m:continue
        c=m.group(1)
        if 'feature = "native"' not in c and 'feature = "server"' not in c:continue
        # test-only cfg(all(test..)) skip counting kind
        # measure item: from next non-attr line until braces balance or ';'
        j=i+1
        while j<len(L) and L[j].strip().startswith("#["):j+=1
        depth=0;started=False;k=j
        while k<len(L):
            depth+=L[k].count("{")-L[k].count("}")
            if "{" in L[k]:started=True
            if (started and depth<=0) or (not started and L[k].rstrip().endswith(";")):break
            k+=1
        n=k-j+1
        key="server" if 'feature = "server"' in c else ("native_neg" if re.search(r'not\(feature = "native"\)',c) else "native_pos")
        tot[key]+=n;detail.append((f,i+1,c,n))
print(tot)
for d in detail:print(d)
