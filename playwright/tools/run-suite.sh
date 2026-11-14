#!/usr/bin/env bash
# Run the whole Playwright suite as duration-balanced CHUNKS, each under its own acquisition of the global pw.lock
# (backlog row 152a). One run of the whole suite holds the lock for hours (debug build, 2 workers: ~100 min), and every
# other lane's spec run waits behind it; chunks of ~15-20 min let the lanes interleave. The chunks are balanced on
# playwright/durations.json (or, before it exists, on test counts from a --list run), and each writes its own JSON
# report, so the per-file table is rebuilt from them with `node playwright/tools/durations.mjs OUT/chunk-*.json`.
#
#   playwright/tools/run-suite.sh --out DIR [--chunks N] [--workers W] [--config suite.local.config.ts]
#                                 [--table FILE] [--lock FILE] [--only-chunk I] [--limit-factor F] [--reuse STORE --tree TREE]
#   needs PLAYWRIGHT_BASE_URL (and PW_BUILD_PROFILE, SSG_SITE_DIR) exported for the build under test, or SSG_SITE_DIR
#   alone (then it serves that dir itself); the server must answer unknown URLs as the site does (tools/site-server.sh).
#
# Every chunk runs under a wall limit (Playwright `--global-timeout`): F x its expected wall time (the chunk's summed
# table seconds / workers, F = 3 by default, at least 900 s). A chunk that hits it ends with its report written: the
# unfinished tests are `skipped` and the report carries "Timed out waiting ...s for the test suite to run" in its
# `errors`, which the deploy's verdict (scripts/deploy-preview.sh `suite_verdict`) counts as a missing chunk, so the
# run is red and the lock is released after minutes instead of held by a wedged chunk.
#
# `--reuse STORE --tree TREE` (backlog row 513) runs only the spec files that have no green record for this tree's site: the
# per-file records of tools/suite-key.mjs, keyed by the site key (every input of the release build) and the spec key (the file,
# what it imports, the config, the data and the environment it reads), and pinned to the exact bytes of the built site
# (`SSG_SITE_DIR` is hashed). TREE must be the worktree's HEAD tree with no tracked change, and `SUITE_BUILD_ENV` carries the
# dx / rustc / cargo versions of the build; when any of that does not hold, or a key cannot be derived, the whole suite runs.
# A run writes one record per spec file it executed (green only when every test of the file passed and its chunk reported
# without run errors); the chunk lines below then cover the files that ran, not the whole suite: the deploy's verdict adds the
# reused files and states them separately.
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
out="" chunks=8 workers=2 config=suite.local.config.ts table="" lock=${LANE_TGT:-/home/user/tgt}/pw.lock only="" factor=3 reuse="" tree=""
while [ $# -gt 0 ]; do
  case "$1" in
    --out) out="$2"; shift 2 ;; --chunks) chunks="$2"; shift 2 ;; --workers) workers="$2"; shift 2 ;;
    --config) config="$2"; shift 2 ;; --table) table="$2"; shift 2 ;; --lock) lock="$2"; shift 2 ;;
    --only-chunk) only="$2"; shift 2 ;; --limit-factor) factor="$2"; shift 2 ;; --reuse) reuse="$2"; shift 2 ;; --tree) tree="$2"; shift 2 ;; *) echo "run-suite: unknown option $1" >&2; exit 2 ;;
  esac
done
[ -n "$out" ] || { echo "run-suite: --out DIR is required" >&2; exit 2; }
mkdir -p "$out"
# Only the timing pass was asked for and it is deactivated: say so before building or serving anything.
if [ "$only" = timing ] && [ "$TIMING_PASS" != on ]; then tools/timing-pass.sh --config "$config"; exit $?; fi
# Row 513: which spec files still need a run. Any doubt (no store or tree, a dirty worktree, another tree, no built site,
# a key that cannot be derived) leaves the whole suite to run.
only_args=()
if [ -n "$reuse" ]; then
  repo="$(cd .. && pwd)"
  if [ -z "$tree" ] || [ -z "${SSG_SITE_DIR:-}" ] || [ ! -d "$SSG_SITE_DIR" ]; then
    echo "run-suite: --reuse needs --tree and SSG_SITE_DIR (a built site); running the whole suite" >&2
  elif [ "$(git -C "$repo" rev-parse "$tree^{tree}" 2>/dev/null)" != "$(git -C "$repo" rev-parse 'HEAD^{tree}')" ] \
       || [ -n "$(git -C "$repo" status --porcelain --untracked-files=no -- . ':(exclude)docs')" ]; then
    echo "run-suite: the worktree is not exactly tree $tree; running the whole suite" >&2
  elif node tools/suite-key.mjs plan --store "$reuse" --tree "$tree" --config "$config" --public "$SSG_SITE_DIR" \
         --build-env "${SUITE_BUILD_ENV:-}" --out "$out/plan.json"; then
    node -e 'console.log(JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")).run.join("\n"))' "$out/plan.json" > "$out/run-files.txt"
    only_args=(--only "$out/run-files.txt")
  else
    echo "run-suite: no key for tree $tree; running the whole suite" >&2
    rm -f "$out/plan.json"
  fi
fi
run_hydration=1
if [ "${#only_args[@]}" -gt 0 ] && ! grep -qx 'oracle/hydration-parity.spec.ts' "$out/run-files.txt"; then run_hydration=0; fi
# hydration-parity.spec.ts only runs the attr-synth binary; build it here, through the build lock, before any
# `--list` or chunk loads the spec (row 338). Not needed when that file is reused.
if [ "$run_hydration" = 1 ]; then tools/ensure-attr-synth.sh || exit 2; fi
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
  node tools/shard.mjs "$i/$chunks" --table "$table" --out "$out/specs-$i.txt" --print-loads ${only_args[@]+"${only_args[@]}"} 2>>"$out/loads.txt"
done
[ "$(cat "$out"/specs-*.txt | grep . | sort -u | wc -l)" = "$(cat "$out"/specs-*.txt | grep . | wc -l)" ] || { echo "run-suite: a spec file is in two chunks" >&2; exit 2; }
total=0
for i in $(seq 1 "$chunks"); do
  [ -n "$only" ] && [ "$only" != "$i" ] && continue
  if [ -z "$(grep . "$out/specs-$i.txt")" ]; then echo "chunk $i/$chunks: no file to run (every file has a green record)"; continue; fi
  start=$(date +%s)
  limit=$(awk -v i="$i" -v n="$chunks" -v w="$workers" -v f="$factor" '$1 == "shard" && $2 == i "/" n ":" { l = f * $5 / w; if (l < 900) l = 900; printf "%d", l + 0.999; exit }' "$out/loads.txt")
  [ -n "$limit" ] || limit=3600
  # A limit that is not a plain number of seconds within a day never reaches Playwright.
  case "$limit" in ''|*[!0-9]*) echo "run-suite: chunk $i has no usable wall limit ('$limit')" >&2; exit 2 ;; esac
  [ "$limit" -le 86400 ] || { echo "run-suite: chunk $i wall limit ${limit}s is over a day" >&2; exit 2; }
  flock "$lock" bash -c '
    s=$(date +%s)
    PW_SPECS_FILE="$1" PLAYWRIGHT_JSON_OUTPUT_FILE="$2" pnpm exec playwright test --config="$3" --workers="$4" --grep-invert @timing --pass-with-no-tests --global-timeout=$(( $7 * 1000 )) --reporter=json,line >"$5" 2>&1
    echo $(( $(date +%s) - s )) > "$6"' _ "$out/specs-$i.txt" "$out/chunk-$i.json" "$config" "$workers" "$out/chunk-$i.log" "$out/chunk-$i.wall" "$limit"
  w=$(cat "$out/chunk-$i.wall" 2>/dev/null || echo 0); total=$((total + w))
  echo "chunk $i/$chunks: $(grep -c . "$out/specs-$i.txt") files, wall_in_lock=${w}s of limit ${limit}s (queued+run $(( $(date +%s) - start ))s), running total ${total}s"
done
if { [ -z "$only" ] || [ "$only" = timing ]; } && [ -n "$(cat "$out"/specs-*.txt | grep .)" ]; then
  cat "$out"/specs-*.txt | grep . > "$out/timing-specs.txt"
  PW_SPECS_FILE="$out/timing-specs.txt" tools/timing-pass.sh --config "$config" --out "$out/chunk-timing" --lock "$lock"
  trc=$?
  if [ "$TIMING_PASS" = on ]; then
    w=$(cat "$out/chunk-timing.wall" 2>/dev/null || echo 0); total=$((total + w))
    echo "timing pass: exit $trc, wall_in_lock=${w}s (one worker, after the quiet-machine wait), running total ${total}s"
  fi
fi
if [ -f "$out/plan.json" ]; then
  node tools/suite-key.mjs record --store "$reuse" --out "$out" --chunks "$chunks"
  echo "run-suite: the chunk lines above cover the $(grep -c . "$out/run-files.txt") spec file(s) that ran; the others were reused from green records (see plan.json)"
fi
echo "run-suite: total wall inside the lock ${total}s"
