#!/usr/bin/env bash
# The one build lock for lanes: a FIFO ticket queue in front of N slots (backlog row 221).
#
#   scripts/lane-lock.sh [gate|lane] [--label TEXT] -- command [args...]
#   scripts/lane-lock.sh --status
#
# Why not bare `flock`: flock wakes a RANDOM waiter, so a lane ready to hand back could lose the
# lock to one mid-work for an hour, and a merge gate had no way to go first. Here every caller
# takes a ticket and only the head of the queue may take a slot.
#
# Queue    $LANE_TGT/lockq/<class>-<arrival-ns>-<pid>, class 0 = gate, 1 = lane (default lane).
#          Order = (class, arrival): a gate jumps every waiting lane, never a running command.
#          A waiter creates its ticket under a temp name, takes an flock on it, THEN renames it into
#          the queue, so every visible ticket is already locked; liveness = "its owner still holds
#          that flock". A killed waiter (or holder) therefore never blocks anyone: the next poll
#          finds its ticket unlocked and removes it.
# Slots    LANE_SLOTS (default 2), each its own lock file lockq/slot-<i>.lock. The head takes the
#          first free slot; slot >= 1 only while MemAvailable >= LANE_MIN_AVAIL_MB (default 5000;
#          one preview rustc/clippy driver is ~2.5 GB, ten concurrent agents were OOM-killed).
#          The command gets LANE_SLOT and LANE_SLOT_SUFFIX ("" for slot 0, "-s1" for slot 1, ...)
#          so it can pick its own target dirs (scripts/lane-slot-target.sh).
# Legacy   Slot 0 ALSO holds LANE_LEGACY_LOCK (default $LANE_TGT/ssg.lock, empty disables) for its
#          whole run, so an old-style `flock ssg.lock cmd` caller and slot 0 exclude each other.
# Fds      The command runs with every lock fd CLOSED, in its own session: a long-lived child
#          (a server, a watcher) cannot keep a slot after the wrapper exits (row 221, the orphaned
#          http.server). TERM/INT/HUP to the wrapper TERMs the command's whole process group
#          (KILL after LANE_KILL_GRACE seconds, default 5) and the wrapper exits with its status.
# Nested   A command that itself calls lane-lock (LANE_LOCK_HELD names this LANE_TGT) just runs.
#
# Known gap: SIGKILL of the wrapper ALONE frees the slot (the kernel drops its flocks) while the
# command, a separate session, may run on. Nothing that holds a lock can survive its own SIGKILL;
# kill the command's process group (`kill -- -<pid>` from --status) rather than the wrapper.
#
# Test/tuning knobs: LANE_POLL (seconds between polls, 0.5), LANE_MEMINFO (a /proc/meminfo stand-in).
set -uo pipefail
export LC_ALL=C
T="${LANE_TGT:-/home/user/tgt}"
Q="$T/lockq"
SLOTS="${LANE_SLOTS:-2}"
MIN_MB="${LANE_MIN_AVAIL_MB:-5000}"
POLL="${LANE_POLL:-0.5}"
GRACE="${LANE_KILL_GRACE:-5}"
LEGACY="${LANE_LEGACY_LOCK-$T/ssg.lock}"
MEMINFO="${LANE_MEMINFO:-/proc/meminfo}"

die() { echo "lane-lock: $*" >&2; exit 2; }
avail_mb() { awk '/^MemAvailable:/ { printf "%d", $2 / 1024; f = 1 } END { if (!f) printf "0" }' "$MEMINFO" 2>/dev/null; }

# held <file>: succeeds iff some process holds an flock on it (never creates the file).
held() {
  local fd
  [ -e "$1" ] || return 1
  { exec {fd}<"$1"; } 2>/dev/null || return 1
  if flock -n "$fd"; then exec {fd}<&-; return 1; fi
  exec {fd}<&-; return 0
}

status() {
  local now i f n pos=0 cls ts base suffix
  now="$(date +%s)"
  echo "lane-lock: $SLOTS slot(s), slot>=1 needs ${MIN_MB} MB available (now $(avail_mb) MB), queue $Q"
  for ((i = 0; i < SLOTS; i++)); do
    f="$Q/slot-$i.lock"
    if held "$f"; then
      n="$(cat "$f" 2>/dev/null)"; ts="${n##*start=}"; ts="${ts%% *}"
      printf 'slot %d: HELD   %s  running %ss\n' "$i" "${n:-?}" "$(( now - ${ts:-$now} ))"
    else
      printf 'slot %d: free\n' "$i"
    fi
  done
  if [ -n "$LEGACY" ] && ! held "$Q/slot-0.lock" && held "$LEGACY"; then
    echo "legacy lock $LEGACY: HELD by an old-style flock caller (slot 0 is closed while it runs)"
  fi
  echo "queue (gate before lane, then arrival):"
  for f in "$Q"/[01]-*; do
    [ -e "$f" ] || continue
    held "$f" || continue
    base="${f##*/}"; cls="${base%%-*}"; ts="${base#*-}"; ts="${ts%%-*}"
    pos=$((pos + 1))
    printf '  %d. %-4s %s  waiting %ss\n' "$pos" "$([ "$cls" = 0 ] && echo gate || echo lane)" "$(cat "$f" 2>/dev/null)" "$(( now - ts / 1000000000 ))"
  done
  [ "$pos" -gt 0 ] || echo "  (empty)"
}

class=lane label=""
while [ $# -gt 0 ]; do
  case "$1" in
    gate|lane) class="$1"; shift ;;
    --label) label="${2:?--label needs a value}"; shift 2 ;;
    --status) status; exit 0 ;;
    --) shift; break ;;
    -h|--help) sed -n '2,36p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) die "unknown argument '$1' (usage: lane-lock.sh [gate|lane] [--label T] -- cmd... | --status)" ;;
  esac
done
[ $# -gt 0 ] || die "no command (usage: lane-lock.sh [gate|lane] [--label T] -- cmd...)"
case "$SLOTS" in ''|*[!0-9]*|0) die "LANE_SLOTS must be a positive integer, got '$SLOTS'" ;; esac

if [ "${LANE_LOCK_HELD:-}" = "$T" ]; then exec "$@"; fi   # nested: the outer call holds the slot

mkdir -p "$Q" || die "cannot create $Q"
[ -z "$LEGACY" ] || mkdir -p "$(dirname "$LEGACY")"

# --- take a ticket: locked first, then published by rename --------------------------------------
tmp="$Q/.tmp.$$"
exec {tfd}>"$tmp" || die "cannot create $tmp"
flock "$tfd"
printf 'pid=%s class=%s label=%s' "$$" "$class" "${label:-$*}" >&"$tfd"
cls=1; [ "$class" = gate ] && cls=0
tk="$Q/$cls-$(date +%s%N)-$$"
mv "$tmp" "$tk"

killer="" child=""
cleanup() {
  [ -z "$killer" ] || kill "$killer" 2>/dev/null
  rm -f "$tmp" "$tk"
}
trap cleanup EXIT
trap 'exit 143' TERM
trap 'exit 130' INT
trap 'exit 129' HUP

# --- wait to be the head of the queue, then for a slot -------------------------------------------
echo "lane-lock: queued as ${class} '${label:-$*}' (pid $$)" >&2
slot="" sfd="" lfd="" announced=0
while :; do
  head=""
  for f in "$Q"/[01]-*; do
    [ -e "$f" ] || continue
    if [ "$f" = "$tk" ] || held "$f"; then head="$f"; break; fi
    rm -f "$f"   # its owner is gone
  done
  if [ "$head" = "$tk" ]; then
    for ((i = 0; i < SLOTS; i++)); do
      if [ "$i" -ge 1 ] && [ "$(avail_mb)" -lt "$MIN_MB" ]; then break; fi
      exec {sfd}>>"$Q/slot-$i.lock"
      if ! flock -n "$sfd"; then exec {sfd}>&-; continue; fi
      if [ "$i" -eq 0 ] && [ -n "$LEGACY" ]; then
        exec {lfd}>>"$LEGACY"
        if ! flock -n "$lfd"; then exec {lfd}>&-; exec {sfd}>&-; continue; fi
      else
        exec {lfd}</dev/null
      fi
      slot="$i"; break
    done
    [ -z "$slot" ] || break
    if [ "$announced" = 0 ]; then echo "lane-lock: head of the queue, waiting for a free slot" >&2; announced=1; fi
  fi
  sleep "$POLL"
done

: > "$Q/slot-$slot.lock"
printf 'pid=%s class=%s label=%s start=%s' "$$" "$class" "${label:-$*}" "$(date +%s)" >&"$sfd"
rm -f "$tk"; exec {tfd}>&-
suffix=""; [ "$slot" -eq 0 ] || suffix="-s$slot"
echo "lane-lock: slot $slot${suffix:+ (suffix $suffix)} acquired" >&2

# --- run: lock fds closed in the command, its own session, signals stop the whole group -----------
export LANE_SLOT="$slot" LANE_SLOT_SUFFIX="$suffix" LANE_LOCK_HELD="$T"
stop_child() {
  [ -n "$child" ] || return 0
  kill -TERM -- "-$child" 2>/dev/null
  if [ -z "$killer" ]; then
    { sleep "$GRACE"; kill -KILL -- "-$child" 2>/dev/null; } {sfd}>&- {lfd}>&- >/dev/null 2>&1 &
    killer=$!
  fi
}
trap stop_child TERM INT HUP
setsid "$@" <&0 {sfd}>&- {lfd}>&- &
child=$!
rc=0
while :; do
  wait "$child"; rc=$?
  kill -0 "$child" 2>/dev/null || break   # still there: wait was interrupted by a signal
done
exit "$rc"
