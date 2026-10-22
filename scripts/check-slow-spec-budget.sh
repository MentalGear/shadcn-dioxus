#!/usr/bin/env bash
#
# check-slow-spec-budget.sh   (backlog row 152a)
#
# The Playwright suite is ~3,500 tests and its cost is TOTAL WORK (a debug full run took ~106 min on two workers
# before row 152a), so it only stays runnable if growth is a decision, not an accident. `playwright/durations.json`
# is the measured per-file table (written by `playwright/tools/durations.mjs` from a JSON-reporter run; it is also what
# `playwright/tools/shard.mjs` balances shards on) and carries the budget in `meta`:
#
#   budget_seconds        the sum of every file's measured seconds may not exceed it
#   max_file_seconds      no single file may exceed it (a file that big needs splitting or trimming first)
#
# This gate is arithmetic over that file and the spec files on disk, with no browser, so it is fast and cannot flake:
#   1. every `*.spec.ts` under playwright/ has an entry (a new spec: run it once with
#      `PLAYWRIGHT_JSON_OUTPUT_FILE=/abs/r.json pnpm exec playwright test --reporter=json <spec>` and
#      `node playwright/tools/durations.mjs /abs/r.json --write playwright/durations.json --merge`);
#   2. no entry names a file that no longer exists (a deleted spec leaves the table in step);
#   3. `meta.total_seconds` equals the sum of the entries (so the budget cannot be dodged by editing only the meta);
#   4. the sum is within `budget_seconds`, and every file within `max_file_seconds`.
# Raising a budget is a visible diff of one number in durations.json, and the commit says why the suite may grow.
#
# Usage: scripts/check-slow-spec-budget.sh      (exit 0 when within budget)

set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 -I - <<'PY'
import json, os, sys

table_path = "playwright/durations.json"
if not os.path.exists(table_path):
    print(f"check-slow-spec-budget: FAIL -- {table_path} is missing (node playwright/tools/durations.mjs <report.json> --write {table_path})")
    sys.exit(1)
table = json.load(open(table_path))
meta, files = table["meta"], table["files"]

on_disk = set()
for root, dirs, names in os.walk("playwright"):
    dirs[:] = [d for d in dirs if d not in ("node_modules", "fixtures", "reference", "tools", "test-results", "playwright-report")]
    for n in names:
        if n.endswith(".spec.ts"):
            on_disk.add(os.path.relpath(os.path.join(root, n), "playwright").replace(os.sep, "/"))

errors = []
for f in sorted(on_disk - set(files)):
    errors.append(f"no entry for {f}: run it and merge (see the header of scripts/check-slow-spec-budget.sh)")
for f in sorted(set(files) - on_disk):
    errors.append(f"stale entry {f}: the spec no longer exists")
total = sum(e["seconds"] for e in files.values())
if abs(round(total) - meta["total_seconds"]) > 1:
    errors.append(f"meta.total_seconds={meta['total_seconds']} but the entries sum to {round(total)}")
budget, cap = meta["budget_seconds"], meta["max_file_seconds"]
if total > budget:
    errors.append(f"suite total {round(total)} s exceeds budget_seconds={budget} (trim or consolidate, or raise the budget with a stated reason)")
for f, e in sorted(files.items(), key=lambda kv: -kv[1]["seconds"]):
    if e["seconds"] > cap:
        errors.append(f"{f} takes {e['seconds']:.0f} s, over max_file_seconds={cap}")
if errors:
    print("check-slow-spec-budget: FAIL")
    for e in errors:
        print("  " + e)
    sys.exit(1)
print(f"check-slow-spec-budget: OK -- {len(files)} spec files, {round(total)} s of {budget} s budget "
      f"({meta.get('build', '?')} build), largest {max(e['seconds'] for e in files.values()):.0f} s of {cap} s")
PY
