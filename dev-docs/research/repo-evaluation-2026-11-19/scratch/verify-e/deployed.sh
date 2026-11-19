cd /tmp/claude-0/-home-user-shadcn-dioxus/2f9af3b0-3995-5f94-a96b-6c0dfde7973a/scratchpad/verify-e/dl-dep
B=https://mentalgear.github.io/shadcn-dioxus
curl -sS -o dep.wasm -w "wasm status=%{http_code} size=%{size_download}\n" $B/assets/preview_bg-dxh92ee26ea6afb8ff3.wasm
sha256sum dep.wasm /home/user/shadcn-dioxus/docs/assets/preview_bg-dxh92ee26ea6afb8ff3.wasm
curl -sSI $B/assets/preview_bg-dxh92ee26ea6afb8ff3.wasm | grep -i 'content-length\|content-encoding\|cache-control\|content-type'
curl -sS -H 'Accept-Encoding: gzip' -o dep.wasm.gz -w "gz-negotiated size=%{size_download}\n" $B/assets/preview_bg-dxh92ee26ea6afb8ff3.wasm
curl -sSI -H 'Accept-Encoding: gzip, br' $B/assets/preview_bg-dxh92ee26ea6afb8ff3.wasm | grep -i 'content-encoding'
: > deppages.txt
while read p; do u="$B/$p/"; [ -z "$p" ] && u="$B/"; r=$(curl -sS -o /tmp/p.html -w "%{http_code} %{size_download}" "$u"); code=${r% *}; sz=$(stat -c%s /tmp/p.html); echo "$code $sz $p" >> deppages.txt; done < /tmp/navpaths.txt
python3 -I -c '
rows=[l.split() for l in open("deppages.txt")]
ok=[int(r[1]) for r in rows if r[0]=="200"]; bad=[r for r in rows if r[0]!="200"]
ok.sort(); n=len(ok)
print("deployed 200 pages",n,"non-200",len(bad),bad[:5],"min/med/max",ok[0],ok[n//2],ok[-1])
'
