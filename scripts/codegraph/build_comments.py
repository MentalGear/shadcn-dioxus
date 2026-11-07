# Comment metrics M1-M5 per file -> comments.json (working-tree text only; no SCIP index needed).
# usage: python3 -I build_comments.py <comments.json>   (stdlib only; the repo root is two levels above this file)
# comments.json: {"fields", "item_fields", "item_kinds", "files": {path: {"cm": [counts], "d": {detail}, "it": [items], "t": 1}}}. The page joins it to the
# graph's nodes by file id, so the graph (index) and the metrics (text) have separate inputs and separate staleness keys.
import sys, os, json, importlib.util
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
spec = importlib.util.spec_from_file_location("comment_metrics", os.path.join(HERE, "comment_metrics.py"))
cmm = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cmm)
out = sys.argv[1]
files = {}
for path, (counts, detail, items) in cmm.measure(REPO, cmm.node_paths(REPO)).items():
    e = {"cm": list(counts)}
    if detail:
        e["d"] = detail
    if items:
        e["it"] = items
    if cmm.is_test_path(path):
        e["t"] = 1
    files[path] = e
with open(out, "w") as fh:
    json.dump({"fields": list(cmm.CM_FIELDS), "item_fields": list(cmm.ITEM_FIELDS), "item_kinds": list(cmm.ITEM_KINDS), "files": files}, fh, separators=(",", ":"))
tot = [sum(e["cm"][i] for e in files.values()) for i in range(len(cmm.CM_FIELDS))]
print("files", len(files), dict(zip(cmm.CM_FIELDS, tot)))
