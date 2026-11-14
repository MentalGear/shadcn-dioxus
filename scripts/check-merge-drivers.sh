#!/usr/bin/env bash
#
# check-merge-drivers.sh   (backlog row 400)
#
# The generated backlog indexes and the durations table never conflict textually on a merge of lane branches: `.gitattributes` maps `dev-docs/backlog.md`, `dev-docs/backlog-done.md` and `playwright/durations.json` to merge drivers (scripts/merge-driver-backlog-index.py, playwright/tools/durations-merge.mjs) that merge by row id and by spec entry, registered per clone by scripts/setup-merge-drivers.sh from the SessionStart hook. This gate checks the attributes, the driver scripts, the registration command and the hook wiring exist, runs both drivers' self-tests, and does a real merge in a throwaway git repo (two branches each adding a backlog row file and a durations entry, registered through setup-merge-drivers.sh itself) that must finish with no conflict and a valid index and table.
#
# This clone's own registration is reported, not required: a clone that never ran scripts/setup-merge-drivers.sh merges with git's text merge (and its conflicts on the generated files), and that script fixes it.
#
# Usage: scripts/check-merge-drivers.sh      Exit status: 0 = OK; 1 = a promise broke.
set -uo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
fail() { echo "check-merge-drivers: FAIL -- $*"; exit 1; }

command -v node >/dev/null || fail "node is required"
command -v python3 >/dev/null || fail "python3 is required"
command -v git >/dev/null || fail "git is required"

# 1. the attributes and the files they name
for want in "dev-docs/backlog.md merge=backlog-index" "dev-docs/backlog-done.md merge=backlog-index" "playwright/durations.json merge=durations"; do
  grep -qxF "$want" .gitattributes 2>/dev/null || fail ".gitattributes has no line '$want'"
done
for f in scripts/merge-driver-backlog-index.py playwright/tools/durations-merge.mjs scripts/setup-merge-drivers.sh; do
  [ -f "$f" ] || fail "$f is missing"
done
grep -q 'setup-merge-drivers.sh' scripts/session-setup.sh || fail "scripts/session-setup.sh does not run scripts/setup-merge-drivers.sh (a clone would never register the drivers)"

# 2. the drivers' own self-tests
python3 -I scripts/merge-driver-backlog-index.py --self-test >/dev/null 2>&1 || { python3 -I scripts/merge-driver-backlog-index.py --self-test; fail "the backlog-index driver's self-test is red"; }
node playwright/tools/durations-merge.mjs --self-test >/dev/null 2>&1 || { node playwright/tools/durations-merge.mjs --self-test; fail "the durations driver's self-test is red"; }

# 3. a real merge in a scratch repo, registered the way a clone is
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
r="$tmp/repo"
git init -q "$r" || fail "git init failed"
git -C "$r" config user.email gate@example.invalid
git -C "$r" config user.name gate
mkdir -p "$r/scripts" "$r/playwright/tools" "$r/dev-docs"
cp scripts/backlog-index.py scripts/merge-driver-backlog-index.py "$r/scripts/"
cp playwright/tools/durations-merge.mjs "$r/playwright/tools/"
cp .gitattributes "$r/"
cp -r dev-docs/backlog "$r/dev-docs/backlog"
cp dev-docs/backlog.md dev-docs/backlog-done.md "$r/dev-docs/"
python3 -I - "$r" <<'PY'
import json, sys
r = sys.argv[1]
e = lambda t, s: {"tests": t, "seconds": s, "passed": t, "failed": 0, "skipped": 0, "retried": 0}
files = {"a.spec.ts": e(1, 10), "b.spec.ts": e(2, 20), "c.spec.ts": e(3, 30)}
meta = {"build": "debug", "tests": 6, "files": 3, "total_seconds": 60, "budget_seconds": 1000, "max_file_seconds": 500}
open(f"{r}/playwright/durations.json", "w").write(json.dumps({"meta": meta, "files": files}, indent=1) + "\n")
PY
"$repo_root/scripts/setup-merge-drivers.sh" --repo "$r" 2>/dev/null || fail "setup-merge-drivers.sh --repo failed"
"$repo_root/scripts/setup-merge-drivers.sh" --repo "$r" --check || fail "setup-merge-drivers.sh is not idempotent"
g() { git -C "$r" "$@"; }
g add -A && g commit -qm base || fail "base commit failed"
base_branch="$(g rev-parse --abbrev-ref HEAD)"

add_row() { # branch id durations-entry-name seconds
  g checkout -q -b "$1" "$base_branch" || fail "checkout $1 failed"
  cat > "$r/dev-docs/backlog/$2-gate-row.md" <<ROW
---
id: $2
title: "Gate row $2"
status: "BACKLOG. gate"
size: "S"
area: tooling
updated: 2026-10-24
---

# Row $2: Gate row $2

## Item

Gate row $2.

## Status / trigger

BACKLOG. gate

## Size

S

## Detail

none
ROW
  (cd "$r" && python3 -I scripts/backlog-index.py >/dev/null) || fail "backlog-index.py failed on branch $1"
  python3 -I - "$r" "$3" "$4" <<'PY'
import json, sys
r, name, secs = sys.argv[1], sys.argv[2], float(sys.argv[3])
p = f"{r}/playwright/durations.json"
t = json.load(open(p))
t["files"][name] = {"tests": 2, "seconds": secs, "passed": 2, "failed": 0, "skipped": 0, "retried": 0}
t["files"]["a.spec.ts" if name.startswith("x") else "b.spec.ts"]["seconds"] += 1   # each branch also re-measures a different entry
v = t["files"].values()
t["meta"].update(tests=sum(x["tests"] for x in v), files=len(t["files"]), total_seconds=round(sum(x["seconds"] for x in v)))
open(p, "w").write(json.dumps(t, indent=1) + "\n")
PY
  g add -A && g commit -qm "branch $1" || fail "commit on $1 failed"
}
add_row left 9001 x-left.spec.ts 7
add_row right 9002 y-right.spec.ts 9
g checkout -q left || fail "checkout left failed"
pre_merge="$(g rev-parse HEAD)"
mout="$(g merge --no-edit right 2>&1)" || { echo "$mout"; fail "merging two branches that each added a backlog row and a durations entry conflicted: the drivers did not run or did not merge"; }

(cd "$r" && python3 -I scripts/backlog-index.py --check) || fail "the merged backlog index does not match the merged row files"
grep -q '^| \[9001\]' "$r/dev-docs/backlog.md" && grep -q '^| \[9002\]' "$r/dev-docs/backlog.md" || fail "the merged index lacks a row"
python3 -I - "$r" <<'PY' || fail "the merged durations table is not valid"
import json, sys
t = json.load(open(sys.argv[1] + "/playwright/durations.json"))
f, m = t["files"], t["meta"]
assert "x-left.spec.ts" in f and "y-right.spec.ts" in f, "an added entry is missing"
assert f["a.spec.ts"]["seconds"] == 11 and f["b.spec.ts"]["seconds"] == 21, "a re-measured entry was lost"
assert m["files"] == len(f) == 5, m
assert m["tests"] == sum(x["tests"] for x in f.values()), m
assert m["total_seconds"] == round(sum(x["seconds"] for x in f.values())), m
assert list(f) == sorted(f), "entries are not sorted"
PY

# the scenario is a real test: without the drivers the same merge conflicts on both generated files
g reset -q --hard "$pre_merge" || fail "reset failed"
g config --remove-section merge.backlog-index && g config --remove-section merge.durations || fail "could not unregister the drivers in the scratch repo"
if g merge --no-edit right >/dev/null 2>&1; then fail "the scratch merge is clean even without the drivers: the scenario does not exercise them"; fi
g merge --abort >/dev/null 2>&1

# 4. this clone's registration (information only)
if "$repo_root/scripts/setup-merge-drivers.sh" --check 2>/dev/null; then
  reg="registered in this clone"
else
  reg="NOT registered in this clone (run scripts/setup-merge-drivers.sh); merges here use git's text merge"
fi
echo "check-merge-drivers: OK -- attributes, scripts and hook wiring present, both self-tests green, a two-branch merge was clean with a valid index and table; $reg"
