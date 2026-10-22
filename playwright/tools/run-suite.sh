#!/usr/bin/env bash
# Run the whole Playwright suite as duration-balanced CHUNKS, each under its own acquisition of the global pw.lock
# (backlog row 152a). One run of the whole suite holds the lock for hours (debug build, 2 workers: ~100 min), and every
# other lane's spec run waits behind it; chunks of ~15-20 min let the lanes interleave. The chunks are balanced on
# playwright/durations.json (or, before it exists, on test counts from a --list run), and each writes its own JSON
# report, so the per-file table is rebuilt from them with `node playwright/tools/durations.mjs OUT/chunk-*.json`.
#
#   playwright/tools/run-suite.sh --out DIR [--chunks N] [--workers W] [--config suite.local.config.ts]
#                                 [--table FILE] [--lock FILE] [--only-chunk I]
#   needs PLAYWRIGHT_BASE_URL (and PW_BUILD_PROFILE, SSG_SITE_DIR) exported for the build under test.
#
# Prints one line per chunk: tests passed/failed, wall seconds INSIDE the lock (queue time excluded), and the sum.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
out="" chunks=8 workers=2 config=suite.local.config.ts table="" lock=${LANE_TGT:-/home/user/tgt}/pw.lock only=""
while [ $# -gt 0 ]; do
  case "$1" in
    --out) out="$2"; shift 2 ;; --chunks) chunks="$2"; shift 2 ;; --workers) workers="$2"; shift 2 ;;
    --config) config="$2"; shift 2 ;; --table) table="$2"; shift 2 ;; --lock) lock="$2"; shift 2 ;;
    --only-chunk) only="$2"; shift 2 ;; *) echo "run-suite: unknown option $1" >&2; exit 2 ;;
  esac
done
[ -n "$out" ] || { echo "run-suite: --out DIR is required" >&2; exit 2; }
mkdir -p "$out"
# hydration-parity.spec.ts only runs the attr-synth binary; build it here, through the build lock, before any
# `--list` or chunk loads the spec (row 338).
tools/ensure-attr-synth.sh || exit 2
if [ -z "$table" ]; then
  if [ -f durations.json ]; then table=durations.json
  else table="$out/list.json"; PLAYWRIGHT_JSON_OUTPUT_FILE="$table" pnpm exec playwright test --config="$config" --list --reporter=json >/dev/null || exit 2; fi
fi
# Snapshot the table and cut ALL the chunks up front: every chunk is a slice of ONE partition. (Cutting chunk i just
# before it runs re-reads durations.json each time; editing the table mid-run then put 34 files in two chunks and 34 in
# none on the first full run, 2026-10-19.)
cp "$table" "$out/table.json"; table="$out/table.json"
: > "$out/loads.txt"
for i in $(seq 1 "$chunks"); do
  node tools/shard.mjs "$i/$chunks" --table "$table" --out "$out/specs-$i.txt" --print-loads 2>>"$out/loads.txt"
done
[ "$(cat "$out"/specs-*.txt | sort -u | wc -l)" = "$(cat "$out"/specs-*.txt | wc -l)" ] || { echo "run-suite: a spec file is in two chunks" >&2; exit 2; }
total=0
for i in $(seq 1 "$chunks"); do
  [ -n "$only" ] && [ "$only" != "$i" ] && continue
  start=$(date +%s)
  flock "$lock" bash -c '
    s=$(date +%s)
    PW_SPECS_FILE="$1" PLAYWRIGHT_JSON_OUTPUT_FILE="$2" pnpm exec playwright test --config="$3" --workers="$4" --reporter=json,line >"$5" 2>&1
    echo $(( $(date +%s) - s )) > "$6"' _ "$out/specs-$i.txt" "$out/chunk-$i.json" "$config" "$workers" "$out/chunk-$i.log" "$out/chunk-$i.wall"
  w=$(cat "$out/chunk-$i.wall" 2>/dev/null || echo 0); total=$((total + w))
  echo "chunk $i/$chunks: $(wc -l < "$out/specs-$i.txt") files, wall_in_lock=${w}s (queued+run $(( $(date +%s) - start ))s), running total ${total}s"
done
echo "run-suite: total wall inside the lock ${total}s"
