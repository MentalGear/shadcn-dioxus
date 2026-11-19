cd /home/user/shadcn-dioxus/docs
W=assets/preview_bg-dxh92ee26ea6afb8ff3.wasm
echo "raw $(stat -c%s $W)"
echo "gzip9 $(gzip -9 -c $W | wc -c)"
node -e 'const z=require("zlib"),fs=require("fs");const b=fs.readFileSync(process.argv[1]);console.log("brotli11",z.brotliCompressSync(b,{params:{[z.constants.BROTLI_PARAM_QUALITY]:11,[z.constants.BROTLI_PARAM_LGWIN]:24,[z.constants.BROTLI_PARAM_SIZE_HINT]:b.length}}).length)' $W
echo "--- js/css assets referenced by index.html"
for f in $(grep -o 'assets/[^"]*\.\(js\|css\)' index.html | sort -u); do echo "$f raw=$(stat -c%s $f) gz=$(gzip -9 -c $f|wc -c)"; done
echo "--- html pages"
find . -name '*.html' | wc -l
find . -name '*.html' -printf '%s %p\n' | sort -n | awk '{a[NR]=$1} END{print "min",a[1],"median",a[int((NR+1)/2)],"max",a[NR],"n",NR}'
find . -name '*.html' -printf '%s %p\n' | sort -n | sed -n '1p;$p'
echo "--- distinct css/js linked per page (count of distinct sets)"
for f in $(find . -name index.html); do grep -o 'assets/[^"]*\.\(js\|css\)' $f | sort -u | tr '\n' ' ' | md5sum; done | sort | uniq -c
