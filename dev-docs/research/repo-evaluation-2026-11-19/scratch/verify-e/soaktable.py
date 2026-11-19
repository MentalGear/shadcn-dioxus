import json,glob,os,sys
V='/tmp/claude-0/-home-user-shadcn-dioxus/2f9af3b0-3995-5f94-a96b-6c0dfde7973a/scratchpad/verify-e'
print('cycle  wasmMB heapMB(after GC) CDPnodes qsaAll listeners   [per log]')
for f in ['soak-local-dialog','soak-local-combobox','soak-local-popover','soak-local-dropdown_menu','soak-deployed-dialog','soak-deployed-combobox','soak-noperf-dialog']:
    rows=[json.loads(l) for l in open(f'{V}/{f}.log') if l.startswith('{"cycle"')]
    print('==',f)
    sel=[r for r in rows if r['cycle'] in (0,100,300,500,700,1000,1300,1500) or r is rows[-1]]
    for r in sel:print(f"{r['cycle']:>5} {r['wasmMB']:>7} {r['heapMB']:>6} {r['nodes']:>6} {r['qsaAll']:>6} {r['listeners']:>5}")
    n=len(rows);xs=[r['cycle'] for r in rows];ys=[r['wasmMB'] for r in rows]
    # slope by least squares KB/cycle and linearity (R^2) and last-half slope vs first-half slope
    import statistics
    def slope(a,b):
        mx=statistics.mean(a);my=statistics.mean(b);return sum((x-mx)*(y-my) for x,y in zip(a,b))/sum((x-mx)**2 for x in a)
    h=n//2
    print(f'wasm slope KB/cycle all {slope(xs,ys)*1024:.2f} first-half {slope(xs[:h+1],ys[:h+1])*1024:.2f} second-half {slope(xs[h:],ys[h:])*1024:.2f}')
