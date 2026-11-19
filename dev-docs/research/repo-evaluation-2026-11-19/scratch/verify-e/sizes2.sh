cd /home/user/shadcn-dioxus/docs
W=assets/preview_bg-dxh92ee26ea6afb8ff3.wasm
python3 -I -c '
import zlib,sys
b=open(sys.argv[1],"rb").read()
for l in (1,6,9): print("zlib level",l,len(zlib.compress(b,l)))
' $W
echo "--- nav-linked pages (hrefs in docs/index.html under /shadcn-dioxus/, non-asset)"
grep -o 'href="/shadcn-dioxus/[^"#]*"' index.html | sed 's/href="\/shadcn-dioxus\///;s/?"$//;s/"$//' | grep -v '^assets/' | sort -u > /tmp/navpaths.txt
wc -l < /tmp/navpaths.txt
n=0; : > /tmp/navsizes.txt
while read p; do f="$p/index.html"; [ -z "$p" ] && f=index.html; if [ -f "$f" ]; then echo "$(stat -c%s $f) $(gzip -9 -c $f|wc -c) $p" >> /tmp/navsizes.txt; else echo "missing $p"; fi; done < /tmp/navpaths.txt
python3 -I -c '
l=[tuple(map(int,x.split()[:2])) for x in open("/tmp/navsizes.txt")]
r=sorted(a for a,b in l); g=sorted(b for a,b in l); n=len(l)
print("nav pages present",n,"raw min/med/max",r[0],r[n//2] if n%2 else (r[n//2-1]+r[n//2])/2,r[-1],"gz min/med/max",g[0],g[n//2],g[-1])
'
echo "--- all index.html under docs"
find . -name 'index.html' | wc -l
find . -name '*.html' ! -name index.html
echo "--- total html"; find . -name '*.html' -printf '%s\n' | awk '{s+=$1} END{print s}'
echo "--- home page request list (static refs in index.html)"; grep -o '\(href\|src\)="/shadcn-dioxus/assets/[^"]*"' index.html | sort -u | wc -l
