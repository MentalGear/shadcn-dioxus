# Comment age vs code age (N5) per file -> blame.json, from `git blame --line-porcelain` output that run.sh --blame cached.
# usage: python3 -I build_blame.py <out dir> <blame.json>   (stdlib only; the repo root is two levels above this file)
# Input: <out>/blame.index lines `<blob hash> <path>` and <out>/blame/<blob hash> (the blame header and author-time lines).
# blame.json: {"now": epoch, "files": {path: [comment_age_sum, comment_lines, code_age_sum, code_lines, comment_median, code_median]}}
# Ages are days since the line's last change. The page sums the sums, so a component's mean is the mean over its lines;
# the median is per file. A shallow clone dates every line older than its boundary commit at the boundary.
import sys, os, re, json, time, importlib.util
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
spec = importlib.util.spec_from_file_location("comment_metrics", os.path.join(HERE, "comment_metrics.py"))
cmm = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cmm)
out, dest = sys.argv[1], sys.argv[2]
HEADER = re.compile(r"^([0-9a-f]{40}) \d+ (\d+)")
now = time.time()


def ages(blame_path):
    """{final line: age in days} from one cached porcelain file."""
    line_sha, times, last = {}, {}, None
    with open(blame_path, encoding="utf-8", errors="replace") as fh:
        for ln in fh:
            m = HEADER.match(ln)
            if m:
                last = m.group(1)
                line_sha[int(m.group(2))] = last
            elif ln.startswith("author-time ") and last:
                times[last] = int(ln.split()[1])
    return {k: max(0.0, (now - times[s]) / 86400) for k, s in line_sha.items() if s in times}


def median(a):
    a = sorted(a)
    return a[len(a) // 2] if a else 0.0


allowed = set(cmm.node_paths(REPO))
files = {}
for entry in open(os.path.join(out, "blame.index"), encoding="utf-8"):
    bh, _, path = entry.rstrip("\n").partition(" ")
    if path not in allowed:
        continue
    bp = os.path.join(out, "blame", bh)
    if not os.path.isfile(bp) or os.path.getsize(bp) == 0:
        continue
    with open(os.path.join(REPO, path), encoding="utf-8", errors="replace") as fh:
        text = fh.read()
    com, code = cmm.line_kinds(path, text)
    age = ages(bp)
    ca = [age[k] for k in com if k in age]
    da = [age[k] for k in code if k in age]
    if ca or da:
        files[path] = [round(sum(ca), 1), len(ca), round(sum(da), 1), len(da), round(median(ca), 1), round(median(da), 1)]
with open(dest, "w") as fh:
    json.dump({"now": int(now), "files": files}, fh, separators=(",", ":"))
n = sum(v[1] for v in files.values())
print("files", len(files), "comment lines", n, "code lines", sum(v[3] for v in files.values()))
