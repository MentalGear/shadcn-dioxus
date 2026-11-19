import json, sys, collections
seen = set()
rows = []
for line in open(sys.argv[1]):
    try:
        m = json.loads(line)
    except Exception:
        continue
    if m.get("reason") != "compiler-message":
        continue
    msg = m["message"]
    code = (msg.get("code") or {}).get("code")
    if not code or not code.startswith("clippy::"):
        continue
    spans = [s for s in msg["spans"] if s["is_primary"]]
    if not spans:
        continue
    s = spans[0]
    pkg = m["package_id"].split("#")[-1].split("@")[0] if "#" in m["package_id"] else m["package_id"]
    pkg = m["package_id"].rsplit("/", 1)[-1].split("#")[0] if "path+" in m["package_id"] else pkg
    key = (code, s["file_name"], s["line_start"], s["column_start"])
    if key in seen:
        continue
    seen.add(key)
    rows.append((code, s["file_name"], s["line_start"], msg["message"], msg["level"]))
def crate(f):
    p = f.split("/")
    return p[0] if p[0] in ("primitives", "preview", "cli", "decision-kernel", "dioxus-attributes", "test-harness", "scripts") else "other"
cnt = collections.defaultdict(collections.Counter)
for code, f, l, msg, lv in rows:
    cnt[code][crate(f)] += 1
crates = ["primitives", "preview", "cli", "decision-kernel", "dioxus-attributes", "test-harness", "scripts", "other"]
print("| lint | " + " | ".join(crates) + " | total |")
print("|---|" + "---|" * (len(crates) + 1))
for code in sorted(cnt, key=lambda c: -sum(cnt[c].values())):
    print(f"| {code} | " + " | ".join(str(cnt[code].get(c, 0)) for c in crates) + f" | {sum(cnt[code].values())} |")
print("\ntotal unique sites:", len(rows))
print("\nSITES")
for code, f, l, msg, lv in sorted(rows):
    print(f"{code}\t{f}:{l}\t{msg[:110]}")
