import json,collections
g=json.load(open('target-codegraph/graph.json'));c=json.load(open('target-codegraph/comments.json'))
n=g['nodes'];e=g['edges']
print('nodes',len(n),'comps',len(g['comps']),'edges',len(e),dict(collections.Counter(x[3] for x in e)))
print('meta',g['meta'])
idx={x['id']:i for i,x in enumerate(n)}
i=idx['primitives/src/lib.rs'];print('lib.rs fanin weight',sum(x[2] for x in e if x[1]==i))
print('node keys',sorted({k for x in n for k in x}))
F=c['files'];print('files',len(F))
t=[sum(f['cm'][j] for f in F.values()) for j in range(14)];print('cm totals',dict(zip(c['fields'],t)))
print('pub doc coverage items+fields',(t[4]+t[6])/(t[3]+t[5]),'undocumented',(t[3]+t[5])-(t[4]+t[6]))
print('und',sum(len(f.get('d',{}).get('und',[])) for f in F.values()),'dup runs',sum(len(f.get('d',{}).get('dup',[])) for f in F.values()))
