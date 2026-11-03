# page template + graph.json + cytoscape.min.js -> one self-contained explorer.html (stdlib only).
# usage: python3 -I make_explorer.py <template> <graph.json> <cytoscape.min.js> <out.html>
import sys
tpl, graph, lib, out = sys.argv[1:5]
t = open(tpl).read()
g = open(graph).read().replace('</', '<\\/')
c = open(lib).read().replace('</script', '<\\/script')
for token in ('__CYTOSCAPE__', '__GRAPH__'):
    assert t.count(token) == 1, token + ' must occur once in the template'
# the library goes in first; a replacement string is never re-scanned, so a token inside the library is harmless
head, rest = t.split('__CYTOSCAPE__')
open(out, 'w').write(head + c + rest.replace('__GRAPH__', g))
