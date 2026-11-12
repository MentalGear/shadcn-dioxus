#!/usr/bin/env bash
# Run the whole Playwright suite as duration-balanced CHUNKS, each under its own acquisition of the global pw.lock
# (backlog row 152a). One run of the whole suite holds the lock for hours (debug build, 2 workers: ~100 min), and every
# other lane's spec run waits behind it; chunks of ~15-20 min let the lanes interleave. The chunks are balanced on
# playwright/durations.json (or, before it exists, on test counts from a --list run), and each writes its own JSON
# report, so the per-file table is rebuilt from them with `node playwright/tools/durations.mjs OUT/chunk-*.json`.
#
#   playwright/tools/run-suite.sh --out DIR [--chunks N] [--workers W] [--config suite.local.config.ts]
#                                 [--table FILE] [--lock FILE] [--only-chunk I] [--limit-factor F]
#   needs PLAYWRIGHT_BASE_URL (and PW_BUILD_PROFILE, SSG_SITE_DIR) exported for the build under test, or SSG_SITE_DIR
#   alone (then it serves that dir itself); the server must answer unknown URLs as the site does (tools/site-server.sh).
#
# Every chunk runs under a wall limit (Playwright `--global-timeout`): F x its expected wall time (the chunk's summed
# table seconds / workers, F = 3 by default, at least 900 s). A chunk that hits it ends with its report written: the
# unfinished tests are `skipped` and the report carries "Timed out waiting ...s for the test suite to run" in its
# `errors`, which the deploy's verdict (scripts/deploy-preview.sh `suite_verdict`) counts as a missing chunk, so the
# run is red and the lock is released after minutes instead of held by a wedged chunk.
#
# Prints one line per chunk: tests passed/failed, wall seconds INSIDE the lock (queue time excluded), and the sum.
#
# The chunks run every test except the `@timing` cases (playwright/timing.ts). Those run last, in ONE more pass
# (tools/timing-pass.sh): one worker, started only when tools/wait-quiet.sh sees the machine idle, under the same lock,
# written to OUT/chunk-timing.json. A timing case measures time, and three workers plus the lane builds of a shared box
# move the measurement more than any code change does. `--only-chunk timing` runs just that pass; `--only-chunk N` runs
# chunk N alone. A pass that never gets a quiet machine writes no report, and the deploy's verdict counts that as red.
# While tools/timing-switch.sh says `off` the pass prints that it is deactivated and writes no report (`TIMING_PASS=on`
# runs it by hand).
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
source tools/timing-switch.sh
out="" chunks=8 workers=2 config=suite.local.config.ts table="" lock=${LANE_TGT:-/home/user/tgt}/pw.lock only="" factor=3
while [ $# -gt 0 ]; do
  case "$1" in
    --out) out="$2"; shift 2 ;; --chunks) chunks="$2"; shift 2 ;; --workers) workers="$2"; shift 2 ;;
    --config) config="$2"; shift 2 ;; --table) table="$2"; shift 2 ;; --lock) lock="$2"; shift 2 ;;
    --only-chunk) only="$2"; shift 2 ;; --limit-factor) factor="$2"; shift 2 ;; *) echo "run-suite: unknown option $1" >&2; exit 2 ;;
  esac
done
[ -n "$out" ] || { echo "run-suite: --out DIR is required" >&2; exit 2; }
mkdir -p "$out"
# Only the timing pass was asked for and it is deactivated: say so before building or serving anything.
if [ "$only" = timing ] && [ "$TIMING_PASS" != on ]; then tools/timing-pass.sh --config "$config"; exit $?; fi
# hydration-parity.spec.ts only runs the attr-synth binary; build it here, through the build lock, before any
# `--list` or chunk loads the spec (row 338).
tools/ensure-attr-synth.sh || exit 2
source tools/site-server.sh
site_server_ready run-suite || exit 2
if [ -z "$table" ]; then
  if [ -f durations.json ]; then table=durations.json
  else table="$out/list.json"; PLAYWRIGHT_JSON_OUTPUT_FILE="$table" pnpm exec playwright test --config="$config" --list --reporter=json >/dev/null || exit 2; fi
fi
# Snapshot the table and cut ALL the chunks up front: every chunk is a slice of ONE partition. (Cutting chunk i just
# before it runs re-reads durations.json each time, so editing the table mid-run could put a file in two chunks
# and in none.)
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
  limit=$(awk -v i="$i" -v n="$chunks" -v w="$workers" -v f="$factor" '$1 == "shard" && $2 == i "/" n ":" { l = f * $5 / w; if (l < 900) l = 900; printf "%d", l + 0.999 }' "$out/loads.txt")
  [ -n "$limit" ] || limit=3600
  flock "$lock" bash -c '
    s=$(date +%s)
    PW_SPECS_FILE="$1" PLAYWRIGHT_JSON_OUTPUT_FILE="$2" pnpm exec playwright test --config="$3" --workers="$4" --grep-invert @timing --pass-with-no-tests --global-timeout=$(( $7 * 1000 )) --reporter=json,line >"$5" 2>&1
    echo $(( $(date +%s) - s )) > "$6"' _ "$out/specs-$i.txt" "$out/chunk-$i.json" "$config" "$workers" "$out/chunk-$i.log" "$out/chunk-$i.wall" "$limit"
  w=$(cat "$out/chunk-$i.wall" 2>/dev/null || echo 0); total=$((total + w))
  echo "chunk $i/$chunks: $(wc -l < "$out/specs-$i.txt") files, wall_in_lock=${w}s of limit ${limit}s (queued+run $(( $(date +%s) - start ))s), running total ${total}s"
done
if [ -z "$only" ] || [ "$only" = timing ]; then
  cat "$out"/specs-*.txt > "$out/timing-specs.txt"
  PW_SPECS_FILE="$out/timing-specs.txt" tools/timing-pass.sh --config "$config" --out "$out/chunk-timing" --lock "$lock"
  trc=$?
  if [ "$TIMING_PASS" = on ]; then
    w=$(cat "$out/chunk-timing.wall" 2>/dev/null || echo 0); total=$((total + w))
    echo "timing pass: exit $trc, wall_in_lock=${w}s (one worker, after the quiet-machine wait), running total ${total}s"
  fi
fi
echo "run-suite: total wall inside the lock ${total}s"
