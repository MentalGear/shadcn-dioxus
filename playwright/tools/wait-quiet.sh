#!/usr/bin/env bash
# Blocks until the machine is quiet enough to measure time on it, or fails after a timeout.
#
#   playwright/tools/wait-quiet.sh [--timeout SECONDS] [--max-busy PERCENT] [--window SECONDS] [--need N]
#
# The timing cases (`@timing`, playwright/timing.ts) assert how long the main thread took, so another process's work
# moves their result: a build, a browser from another run, the previous pass's own browsers still exiting. Quiet means,
# N times in a row (default 3):
#   - no build slot is held and no build waits in the queue (`scripts/lane-lock.sh --status`), and
#   - the CPUs were busy at most PERCENT (default 15) of the last WINDOW seconds (default 5), read from /proc/stat
#     (the 1-minute load average is not used: it still shows the previous pass for a minute after it ended).
# On timeout (default 1800 s) it prints what it last saw and exits 1: the caller fails, a timing case is never skipped.
# Environment: TIMING_QUIET_TIMEOUT, TIMING_QUIET_MAX_BUSY, TIMING_QUIET_WINDOW, TIMING_QUIET_NEED (the flags win).
set -uo pipefail
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
timeout="${TIMING_QUIET_TIMEOUT:-1800}" max_busy="${TIMING_QUIET_MAX_BUSY:-15}" window="${TIMING_QUIET_WINDOW:-5}" need="${TIMING_QUIET_NEED:-3}"
while [ $# -gt 0 ]; do
  case "$1" in
    --timeout) timeout="$2"; shift 2 ;; --max-busy) max_busy="$2"; shift 2 ;; --window) window="$2"; shift 2 ;;
    --need) need="$2"; shift 2 ;; *) echo "wait-quiet: unknown option $1" >&2; exit 2 ;;
  esac
done
cpu_sample() { awk '/^cpu /{idle=$5+$6; total=0; for(i=2;i<=NF;i++) total+=$i; print idle, total; exit}' /proc/stat; }
builds_busy() {
  local out; out="$("$repo/scripts/lane-lock.sh" --status 2>/dev/null)" || return 1
  local held waiting
  held="$(sed -n '/: HELD/{p;q}' <<<"$out")"
  [ -z "$held" ] || { echo "$held"; return 0; }
  waiting="$(sed -nE '/^  [^(]/{p;q}' <<<"$out")"
  [ -z "$waiting" ] || { echo "queued: $waiting"; return 0; }
  return 1
}
start=$(date +%s) streak=0 last="" announced=0
while :; do
  read -r i0 t0 < <(cpu_sample); sleep "$window"; read -r i1 t1 < <(cpu_sample)
  busy=$(awk -v i0="$i0" -v i1="$i1" -v t0="$t0" -v t1="$t1" 'BEGIN{d=t1-t0; if(d<=0){print 100; exit} printf "%.0f", 100*(1-(i1-i0)/d)}')
  if holder="$(builds_busy)"; then last="a build holds the lock: $holder"; streak=0
  elif [ "$busy" -gt "${max_busy%.*}" ]; then last="CPU ${busy}% busy over ${window}s (limit ${max_busy}%)"; streak=0
  else streak=$((streak + 1)); last="CPU ${busy}% busy, no build"; fi
  if [ "$streak" -ge "$need" ]; then echo "wait-quiet: quiet after $(( $(date +%s) - start )) s ($last)"; exit 0; fi
  if [ "$announced" = 0 ] && [ "$streak" = 0 ]; then echo "wait-quiet: waiting for a quiet machine before the timing cases: $last"; announced=1; fi
  if [ $(( $(date +%s) - start )) -ge "$timeout" ]; then
    echo "wait-quiet: the machine was not quiet within ${timeout} s (last: $last). The timing cases did NOT run; rerun when nothing else is building (TIMING_QUIET_TIMEOUT to wait longer)." >&2
    exit 1
  fi
done
