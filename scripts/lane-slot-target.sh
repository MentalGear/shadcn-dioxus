#!/usr/bin/env bash
# Per-slot cargo target dirs for scripts/lane-lock.sh (backlog row 221).
#
#   scripts/lane-slot-target.sh dir <name>          print $LANE_TGT/<name><LANE_SLOT_SUFFIX>, creating it
#   scripts/lane-slot-target.sh check-disk <dir>    exit 1 unless free disk - live reservations >= size(dir) + 20% + 2 GB
#   scripts/lane-slot-target.sh reserve <dir>       the same check, then records a reservation for the caller's pid
#   scripts/lane-slot-target.sh release             drops it (also dropped when the pid dies)
#   scripts/lane-slot-target.sh copy <src> <dest>   copy of src without .tree-owner and without `incremental/` dirs
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

# Disk guard as a RESERVATION (row 356): two copies that start together must not both pass.
# Under one flock on $Q/disk.lock: free space minus the bytes live copies already reserved
# ($Q/disk-reserve/<pid>, one file per owner pid, ignored and removed when that pid is dead) must
# cover this copy; the reservation is written before the lock is released and dropped by `release`.
# Row 367: a copy never carries cargo's `incremental/` dirs (3.0 of the template's 6.8 GB): every lane command runs
# with CARGO_INCREMENTAL=0 (lane-cargo.sh, lane-ssg.sh, run-gates.sh), which neither reads nor writes them, so they
# were dead weight that made each lane build need 10.4 GB free. The guard's size estimate leaves them out too
# (`du --exclude`). LANE_COPY_INCREMENTAL=1 keeps them (for a hand-run incremental build).
# Test hooks: LANE_FREE_KB (fake free space), LANE_DISK_MARGIN_KB (default 2 GB), LANE_RESERVE_PID.
RES="$Q/disk-reserve"
live_reserved_kb() { # sums live reservations, deletes dead ones; call under the disk lock
  local f pid sum=0 kb
  for f in "$RES"/*; do
    [ -e "$f" ] || continue
    pid="${f##*/}"
    if kill -0 "$pid" 2>/dev/null; then kb="$(cat "$f" 2>/dev/null)"; sum=$(( sum + ${kb:-0} )); else rm -f "$f"; fi
  done
  echo "$sum"
}
slim_du_args() { [ "${LANE_COPY_INCREMENTAL:-0}" = 1 ] || echo "--exclude=incremental"; }
check_disk() { # $1 = dir whose size is the estimate; $2 = "reserve" to record the reservation for $RESERVE_PID
  local need_kb free_kb held_kb
  need_kb="$(du -sk $(slim_du_args) "$1" 2>/dev/null | cut -f1)"; need_kb="${need_kb:-0}"
  need_kb=$(( need_kb + need_kb / 5 + ${LANE_DISK_MARGIN_KB:-2097152} ))
  mkdir -p "$RES"
  exec 202>>"$Q/disk.lock"; flock 202
  held_kb="$(live_reserved_kb)"
  free_kb="${LANE_FREE_KB:-$(df -Pk "$T" | awk 'NR==2 { print $4 }')}"
  if [ $(( free_kb - held_kb )) -lt "$need_kb" ]; then
    flock -u 202; exec 202>&-
    die "refusing: $((free_kb / 1024)) MiB free under $T, $((held_kb / 1024)) MiB reserved by running copies, need $((need_kb / 1024)) MiB (size of $1 + 20% + 2 GB)"
  fi
  if [ "${2:-}" = reserve ]; then echo "$need_kb" > "$RES/$RESERVE_PID"; fi
  flock -u 202; exec 202>&-
}
release() { rm -f "$RES/$RESERVE_PID"; }

# copy <src> <dest>: a warm copy that never carries the source's .tree-owner, so a partial dest
# (ENOSPC, kill) can not name another worktree and look like a stale foreign tree (row 356).
copy_tree() {
  mkdir -p "$2"
  if [ "${LANE_COPY_INCREMENTAL:-0}" = 1 ]; then
    find "$1" -mindepth 1 -maxdepth 1 ! -name .tree-owner -exec cp -a --reflink=auto -t "$2" {} +
  else
    # tar keeps modes, mtimes (cargo freshness) and hard links; the excludes are anchored to target/<profile|triple>[/<profile>]
    tar -C "$1" -cf - --exclude=./.tree-owner --exclude='./*/incremental' --exclude='./*/*/incremental' . | tar -C "$2" -xpf -
  fi
}

cmd="${1:-}"; shift || true
RESERVE_PID="${LANE_RESERVE_PID:-$PPID}"
case "$cmd" in
  check-disk) check_disk "${1:?dir}" ;;
  reserve) check_disk "${1:?dir}" reserve ;;   # reservation is owned by $LANE_RESERVE_PID (default: the caller)
  release) release ;;
  copy) copy_tree "${1:?src}" "${2:?dest}" ;;
  dir)
    name="${1:?name}"; suffix="${LANE_SLOT_SUFFIX:-}"
    src="$T/$name"; dest="$T/$name$suffix"
    if [ -z "$suffix" ] || [ -d "$dest" ]; then mkdir -p "$dest"; echo "$dest"; exit 0; fi
    RESERVE_PID=$$; trap release EXIT
    if [ -d "$src" ]; then check_disk "$src" reserve; elif [ -d "$T/template" ]; then check_disk "$T/template" reserve; fi
    tmp="$dest.tmp.$$"; rm -rf "$tmp"
    mkdir -p "$Q"
    warm=0
    if [ -d "$src" ]; then
      # Warm only if slot 0 is idle; hold its lock (and the legacy lock) while copying.
      if ( exec 200>>"$Q/slot-0.lock"; flock -n 200 || exit 3
           if [ -n "$LEGACY" ]; then exec 201>>"$LEGACY"; flock -n 201 || exit 3; fi
           copy_tree "$src" "$tmp" ) 2>/dev/null; then warm=1; else rm -rf "$tmp"; fi
    fi
    if [ "$warm" = 1 ]; then rm -f "$tmp/.tree-owner"; mv "$tmp" "$dest"; echo "lane-slot-target: $dest warm copy of $src" >&2
    else mkdir -p "$dest"; echo "lane-slot-target: $dest cold (slot 0 busy or $src missing)" >&2; fi
    echo "$dest"
    ;;
  *) sed -n '2,16p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//' >&2; exit 2 ;;
esac
