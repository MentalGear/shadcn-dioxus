# page template + graph.json + comments.json + blame.json + status.json + cytoscape.min.js -> one self-contained explorer.html (stdlib only).
# usage: python3 -I make_explorer.py <template> <graph.json> <comments.json> <blame.json> <status.json> <cytoscape.min.js> <out.html>
import sys
tpl, graph, comments, blame, status, lib, out = sys.argv[1:8]
t = open(tpl).read()
data = {'__GRAPH__': graph, '__COMMENTS__': comments, '__BLAME__': blame, '__STATUS__': status}
for token in ('__CYTOSCAPE__', *data):
    assert t.count(token) == 1, token + ' must occur once in the template'
c = open(lib).read().replace('</script', '<\\/script')
# the library goes in first; a replacement string is never re-scanned, so a token inside the library is harmless
head, rest = t.split('__CYTOSCAPE__')
for token, path in data.items():
    rest = rest.replace(token, open(path).read().replace('</', '<\\/'))
open(out, 'w').write(head + c + rest)
