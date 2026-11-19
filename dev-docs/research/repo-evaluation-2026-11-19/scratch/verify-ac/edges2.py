import json
g=json.load(open('target-codegraph/graph.json'));n=g['nodes'];e=g['edges'];idx={x['id']:i for i,x in enumerate(n)}
for a in ['scripts/attr-spread-collision-checker/src/bin/public-api-snapshot.rs','primitives/src/sleep.rs']:
    print('FROM',a)
    for x in e:
        if x[0]==idx[a]: print('  ->',n[x[1]]['id'],x[2],x[3])
