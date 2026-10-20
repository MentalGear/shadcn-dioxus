#!/usr/bin/env bash
# Per-slot cargo target dirs for scripts/lane-lock.sh (backlog row 221).
#
#   scripts/lane-slot-target.sh dir <name>          print $LANE_TGT/<name><LANE_SLOT_SUFFIX>, creating it
#   scripts/lane-slot-target.sh check-disk <dir>    exit 1 unless free disk >= size(dir) + 20% + 2 GB
#
# Slot 0 uses $LANE_TGT/<name> (check, gates, ...), exactly as before. A slot >= 1 compiles
# at the same time as slot 0 and so cannot share its target dir; its dir (<name>-s1) is created
# lazily, on first use, as a WARM copy of the slot-0 dir when that is idle (the copy runs holding
# slot 0's lock and the legacy lock, so nothing builds into the source meanwhile) and as an empty
# COLD dir otherwise. Either way it refuses (exit 1) when free disk is below the source size + 20% +
# 2 GB: a full disk killed lanes before (row 168). Run it INSIDE lane-lock.sh (LANE_SLOT_SUFFIX).
set -euo pipefail
T="${LANE_TGT:-/home/user/tgt}"
Q="$T/lockq"
LEGACY="${LANE_LEGACY_LOCK-$T/ssg.lock}"
die() { echo "lane-slot-target: $*" >&2; exit 1; }

check_disk() { # $1 = dir whose size is the estimate
  local need_kb free_kb
  need_kb="$(du -sk "$1" 2>/dev/null | cut -f1)"; need_kb="${need_kb:-0}"
  need_kb=$(( need_kb + need_kb / 5 + 2 * 1024 * 1024 ))
  free_kb="$(df -Pk "$T" | awk 'NR==2 { print $4 }')"
  [ "$free_kb" -ge "$need_kb" ] || die "refusing: $((free_kb / 1024)) MiB free under $T, need $((need_kb / 1024)) MiB (size of $1 + 20% + 2 GB)"
}

cmd="${1:-}"; shift || true
case "$cmd" in
  check-disk) check_disk "${1:?dir}" ;;
  dir)
    name="${1:?name}"; suffix="${LANE_SLOT_SUFFIX:-}"
    src="$T/$name"; dest="$T/$name$suffix"
    if [ -z "$suffix" ] || [ -d "$dest" ]; then mkdir -p "$dest"; echo "$dest"; exit 0; fi
    if [ -d "$src" ]; then check_disk "$src"; elif [ -d "$T/template" ]; then check_disk "$T/template"; fi
    tmp="$dest.tmp.$$"; rm -rf "$tmp"
    mkdir -p "$Q"
    warm=0
    if [ -d "$src" ]; then
      # Warm only if slot 0 is idle; hold its lock (and the legacy lock) while copying.
      if ( exec 200>>"$Q/slot-0.lock"; flock -n 200 || exit 3
           if [ -n "$LEGACY" ]; then exec 201>>"$LEGACY"; flock -n 201 || exit 3; fi
           cp -a --reflink=auto "$src" "$tmp" ) 2>/dev/null; then warm=1; else rm -rf "$tmp"; fi
    fi
    if [ "$warm" = 1 ]; then rm -f "$tmp/.tree-owner"; mv "$tmp" "$dest"; echo "lane-slot-target: $dest warm copy of $src" >&2
    else mkdir -p "$dest"; echo "lane-slot-target: $dest cold (slot 0 busy or $src missing)" >&2; fi
    echo "$dest"
    ;;
  *) sed -n '2,13p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//' >&2; exit 2 ;;
esac
