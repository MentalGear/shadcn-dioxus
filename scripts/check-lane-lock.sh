#!/usr/bin/env bash
# Gate: scripts/lane-lock.sh (the FIFO ticket queue in front of the build slots, backlog row 221)
# keeps the properties lane-rules.md promises. A deterministic shell test, no network, no build,
# a few seconds; every scenario runs in its own throwaway LANE_TGT with a short poll interval.
#
#   1 fifo           gate jumps waiting lanes; lanes keep arrival order
#   2 killed-holder  SIGKILL of the wrapper frees the slot (the documented gap: its command runs on)
#   3 killed-waiter  a SIGKILLed waiter's ticket never blocks the queue
#   4 slots-memory   two slots give distinct LANE_SLOT/LANE_SLOT_SUFFIX; low MemAvailable keeps slot 1 closed
#   5 no-fd-leak     a long-lived child of the command holds no lock fd and no slot
#   6 term           TERM to the wrapper stops the command's whole process group
#   7 status-env     exit status and LANE_SLOT/LANE_SLOT_SUFFIX pass through
#   8 legacy         `flock ssg.lock` callers and slot 0 exclude each other, both ways
#   9 disk-reserve   (row 356) two concurrent copies on a disk sized for one: the second refuses; a dead
#                    pid's reservation is ignored; `copy` never carries the source's .tree-owner
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
LL="$PWD/scripts/lane-lock.sh"
work="$(mktemp -d)"
fails=0
pids=()
unset LANE_LOCK_HELD
export LANE_POLL=0.05 LANE_KILL_GRACE=1

cleanup() {
  local p
  for p in "${pids[@]}" $(cat "$work"/*/pids 2>/dev/null); do kill -9 -- "-$p" 2>/dev/null; kill -9 "$p" 2>/dev/null; done
  rm -rf "$work"
}
trap cleanup EXIT

bad() { echo "check-lane-lock: FAIL [$scn] $*" >&2; fails=$((fails + 1)); }
waitfor() { # waitfor <seconds> <cmd...>
  local deadline=$(( $(date +%s%N) + $1 * 1000000000 )); shift
  until "$@"; do [ "$(date +%s%N)" -lt "$deadline" ] || return 1; sleep 0.05; done
}
scenario() { # scenario <name> -> fresh LANE_TGT
  scn="$1"; export LANE_TGT="$work/$1"; rm -rf "$LANE_TGT"; mkdir -p "$LANE_TGT"
  export LANE_SLOTS=1 LANE_MIN_AVAIL_MB=0 LANE_LEGACY_LOCK="$LANE_TGT/ssg.lock"
  unset LANE_MEMINFO
}
queued() { local n=0 f; for f in "$LANE_TGT"/lockq/[01]-*; do [ -e "$f" ] && n=$((n + 1)); done; [ "$n" -ge "$1" ]; }
exists() { [ -e "$1" ]; }
lines() { [ -e "$1" ] && [ "$(wc -l < "$1")" -ge "$2" ]; }
dead() { ! kill -0 "$1" 2>/dev/null; }
holder() { # a command that records its pid and runs until $LANE_TGT/release exists
  echo 'echo $$ >> "$LANE_TGT/pids"; while [ ! -e "$LANE_TGT/release" ]; do sleep 0.05; done'
}

# --- 1 fifo ---------------------------------------------------------------------------------------
scenario fifo
"$LL" --label H -- sh -c "$(holder)" 2>/dev/null & pids+=($!)
waitfor 3 exists "$LANE_TGT/pids" || bad "holder never started"
mk() { "$LL" "$1" --label "$2" -- sh -c "echo $2 >> \"$LANE_TGT/order\"" 2>/dev/null & pids+=($!); waitfor 3 queued "$3" || bad "$2 never queued"; }
mk lane L1 1; mk lane L2 2; mk gate G1 3; mk lane L3 4; mk gate G2 5
st="$("$LL" --status 2>&1)"
[ "$(grep -cE '^ +[0-9]+\. ' <<<"$st")" = 5 ] || bad "--status should list 5 queued, got: $st"
first="$(grep -E '^ +1\. ' <<<"$st")"; [[ "$first" == *gate*G1* ]] || bad "--status head should be gate G1: $first"
touch "$LANE_TGT/release"
waitfor 5 lines "$LANE_TGT/order" 5 || bad "waiters did not all run"
[ "$(tr '\n' ' ' < "$LANE_TGT/order")" = "G1 G2 L1 L2 L3 " ] || bad "order was '$(tr '\n' ' ' < "$LANE_TGT/order")', want 'G1 G2 L1 L2 L3 '"
wait

# --- 2 killed holder ------------------------------------------------------------------------------
scenario killed-holder
"$LL" --label H -- sh -c 'echo $PPID > "$LANE_TGT/wrapper"; '"$(holder)" 2>/dev/null & pids+=($!); disown
waitfor 3 exists "$LANE_TGT/pids" || bad "holder never started"
"$LL" --label W -- sh -c "echo ran > \"$LANE_TGT/w\"" 2>/dev/null & pids+=($!)
waitfor 3 queued 1 || bad "waiter never queued"
kill -9 "$(cat "$LANE_TGT/wrapper")"
waitfor 5 exists "$LANE_TGT/w" || bad "SIGKILL of the holder's wrapper did not free the slot"
cmdpid="$(head -1 "$LANE_TGT/pids")"
kill -0 "$cmdpid" 2>/dev/null || bad "documented gap changed: the command should outlive a SIGKILLed wrapper (update the header comment if this is now fixed)"
touch "$LANE_TGT/release"

# --- 3 killed waiter ------------------------------------------------------------------------------
scenario killed-waiter
"$LL" --label H -- sh -c "$(holder)" 2>/dev/null & pids+=($!)
waitfor 3 exists "$LANE_TGT/pids" || bad "holder never started"
"$LL" --label W1 -- sh -c "echo w1 >> \"$LANE_TGT/order\"" 2>/dev/null & w1=$!; pids+=($w1)
waitfor 3 queued 1 || bad "W1 never queued"
"$LL" --label W2 -- sh -c "echo w2 >> \"$LANE_TGT/order\"" 2>/dev/null & pids+=($!)
waitfor 3 queued 2 || bad "W2 never queued"
kill -9 "$w1"; wait "$w1" 2>/dev/null
touch "$LANE_TGT/release"
waitfor 5 exists "$LANE_TGT/order" || bad "a SIGKILLed waiter at the head blocked the queue"
[ "$(cat "$LANE_TGT/order" 2>/dev/null)" = "w2" ] || bad "order was '$(cat "$LANE_TGT/order" 2>/dev/null)', want only 'w2'"
wait

# --- 4 slots and the memory guard -----------------------------------------------------------------
scenario slots-memory
export LANE_SLOTS=2 LANE_MEMINFO="$LANE_TGT/meminfo" LANE_MIN_AVAIL_MB=5000
printf 'MemTotal: 16000000 kB\nMemAvailable: 1000000 kB\n' > "$LANE_MEMINFO"
rec='echo "$LANE_SLOT:$LANE_SLOT_SUFFIX" >> "$LANE_TGT/slots"; echo $$ >> "$LANE_TGT/pids"; while [ ! -e "$LANE_TGT/release" ]; do sleep 0.05; done'
"$LL" --label A -- sh -c "$rec" 2>/dev/null & pids+=($!)
waitfor 3 exists "$LANE_TGT/slots" || bad "A never started"
"$LL" --label B -- sh -c "$rec" 2>/dev/null & pids+=($!)
waitfor 3 queued 1 || bad "B never queued"
sleep 0.4
[ "$(wc -l < "$LANE_TGT/slots")" = 1 ] || bad "memory guard: B took slot 1 with only 1000 MB available"
printf 'MemTotal: 16000000 kB\nMemAvailable: 9000000 kB\n' > "$LANE_MEMINFO"
waitfor 3 lines "$LANE_TGT/slots" 2 || bad "B did not take slot 1 once memory was available"
[ "$(sort "$LANE_TGT/slots" | tr '\n' ' ')" = "0: 1:-s1 " ] || bad "slots were '$(sort "$LANE_TGT/slots" | tr '\n' ' ')', want '0: 1:-s1 '"
touch "$LANE_TGT/release"; wait
export LANE_MIN_AVAIL_MB=0

# --- 5 no fd leak ---------------------------------------------------------------------------------
scenario no-fd-leak
"$LL" -- sh -c 'sleep 30 >/dev/null 2>&1 & echo $! >> "$LANE_TGT/pids"; echo $! > "$LANE_TGT/sleeper"' 2>/dev/null
sp="$(cat "$LANE_TGT/sleeper" 2>/dev/null)"
[ -n "$sp" ] && kill -0 "$sp" 2>/dev/null || bad "the long-lived child is gone"
if ls -l /proc/"$sp"/fd 2>/dev/null | grep -qE 'lockq|ssg\.lock'; then bad "the child holds a lock fd: $(ls -l /proc/"$sp"/fd | grep -E 'lockq|ssg.lock')"; fi
"$LL" -- sh -c "echo second > \"$LANE_TGT/second\"" 2>/dev/null & pids+=($!)
waitfor 3 exists "$LANE_TGT/second" || bad "a long-lived child kept the slot after the wrapper exited"
wait %% 2>/dev/null

# --- 6 TERM ---------------------------------------------------------------------------------------
scenario term
"$LL" -- sh -c 'sleep 60 & echo $! > "$LANE_TGT/sleeper"; echo $$ >> "$LANE_TGT/pids"; wait' 2>/dev/null & wp=$!; pids+=($wp)
waitfor 3 exists "$LANE_TGT/sleeper" || bad "command never started"
sleep 0.1; sp="$(cat "$LANE_TGT/sleeper")"
kill -TERM "$wp"
waitfor 4 dead "$wp" || bad "wrapper still alive 4 s after TERM"
waitfor 3 dead "$sp" || bad "TERM did not stop the command's child"
wait "$wp" 2>/dev/null; rc=$?; [ "$rc" = 143 ] || bad "wrapper exit status after TERM was $rc, want 143"
"$LL" -- true 2>/dev/null || bad "slot still held after TERM"

# --- 7 exit status and env pass through -----------------------------------------------------------
scenario status-env
"$LL" -- sh -c 'echo "$LANE_SLOT|$LANE_SLOT_SUFFIX|$1" > "$LANE_TGT/env"; exit 42' _ "a b" 2>/dev/null; rc=$?
[ "$rc" = 42 ] || bad "exit status was $rc, want 42"
[ "$(cat "$LANE_TGT/env" 2>/dev/null)" = "0||a b" ] || bad "env/args were '$(cat "$LANE_TGT/env" 2>/dev/null)', want '0||a b'"
"$LL" -- false 2>/dev/null; [ $? = 1 ] || bad "false should give 1"
"$LL" 2>/dev/null; [ $? = 2 ] || bad "no command should be a usage error (2)"

# --- 8 legacy ssg.lock interop --------------------------------------------------------------------
scenario legacy
flock "$LANE_TGT/ssg.lock" sleep 1.0 & lp=$!; pids+=($lp)
waitfor 2 bash -c "! flock -n '$LANE_TGT/ssg.lock' true"
t0=$(date +%s%N)
"$LL" -- sh -c "echo ran > \"$LANE_TGT/ran\"" 2>/dev/null
dt=$(( ($(date +%s%N) - t0) / 1000000 ))
[ -e "$LANE_TGT/ran" ] || bad "queued command never ran after the legacy holder left"
[ "$dt" -ge 500 ] || bad "slot 0 ran while an old-style flock held ssg.lock (${dt} ms)"
"$LL" -- sh -c 'flock -n "$LANE_TGT/ssg.lock" true && echo free > "$LANE_TGT/seen" || echo held > "$LANE_TGT/seen"' 2>/dev/null
[ "$(cat "$LANE_TGT/seen")" = held ] || bad "an old-style flock -n got ssg.lock while slot 0 was running"
flock -n "$LANE_TGT/ssg.lock" true || bad "ssg.lock still held after the command ended"
LANE_LEGACY_LOCK= "$LL" -- sh -c 'flock -n "$LANE_TGT/ssg.lock" true && echo free > "$LANE_TGT/seen2"' 2>/dev/null
[ "$(cat "$LANE_TGT/seen2" 2>/dev/null)" = free ] || bad "LANE_LEGACY_LOCK= should leave ssg.lock alone"

# --- 9 disk reservation (row 356) -----------------------------------------------------------------
scenario disk-reserve
LST="$PWD/scripts/lane-slot-target.sh"
mkdir -p "$LANE_TGT/src"; head -c 102400 /dev/zero > "$LANE_TGT/src/blob"; echo owner > "$LANE_TGT/src/.tree-owner"
export LANE_DISK_MARGIN_KB=0
need=$(( $(du -sk "$LANE_TGT/src" | cut -f1) )); need=$(( need + need / 5 ))
export LANE_FREE_KB=$(( need + need / 2 ))          # room for one copy, not two
sleep 30 & pa=$!; pids+=($pa); sleep 30 & pb=$!; pids+=($pb)
LANE_RESERVE_PID=$pa "$LST" reserve "$LANE_TGT/src" 2>/dev/null || bad "first reservation should pass on a disk sized for one"
LANE_RESERVE_PID=$pb "$LST" reserve "$LANE_TGT/src" 2>/dev/null && bad "second concurrent reservation passed on a disk sized for one"
[ -e "$LANE_TGT/lockq/disk-reserve/$pb" ] && bad "a refused copy left a reservation"
LANE_RESERVE_PID=$pa "$LST" release 2>/dev/null
LANE_RESERVE_PID=$pb "$LST" reserve "$LANE_TGT/src" 2>/dev/null || bad "reservation should pass again once the first copy released"
LANE_RESERVE_PID=$pb "$LST" release 2>/dev/null
# a dead pid's reservation is ignored (and swept)
sleep 0 & dp=$!; wait "$dp"
mkdir -p "$LANE_TGT/lockq/disk-reserve"; echo "$need" > "$LANE_TGT/lockq/disk-reserve/$dp"
LANE_RESERVE_PID=$pa "$LST" reserve "$LANE_TGT/src" 2>/dev/null || bad "a dead pid's reservation blocked a new copy"
[ -e "$LANE_TGT/lockq/disk-reserve/$dp" ] && bad "the dead pid's reservation file was not swept"
LANE_RESERVE_PID=$pa "$LST" release 2>/dev/null
# a partial or complete copy never carries .tree-owner
"$LST" copy "$LANE_TGT/src" "$LANE_TGT/dst" 2>/dev/null
[ -e "$LANE_TGT/dst/blob" ] || bad "copy lost the files"
[ -e "$LANE_TGT/dst/.tree-owner" ] && bad "copy carried the source's .tree-owner"
kill "$pa" "$pb" 2>/dev/null
unset LANE_FREE_KB LANE_DISK_MARGIN_KB

# --- 10 a copy leaves cargo's incremental dirs out (row 367) --------------------------------------
scenario slim-copy
mkdir -p "$LANE_TGT/slim/x86_64-unknown-linux-gnu/server-dev/incremental/preview-1" "$LANE_TGT/slim/x86_64-unknown-linux-gnu/server-dev/deps" "$LANE_TGT/slim/debug/incremental"
head -c 1048576 /dev/zero > "$LANE_TGT/slim/x86_64-unknown-linux-gnu/server-dev/incremental/preview-1/dep-graph.bin"
head -c 1048576 /dev/zero > "$LANE_TGT/slim/debug/incremental/big"
echo lib > "$LANE_TGT/slim/x86_64-unknown-linux-gnu/server-dev/deps/libx.rlib"; echo owner > "$LANE_TGT/slim/.tree-owner"
ln "$LANE_TGT/slim/x86_64-unknown-linux-gnu/server-dev/deps/libx.rlib" "$LANE_TGT/slim/x86_64-unknown-linux-gnu/server-dev/deps/libx-link.rlib"
touch -d '2020-01-02 03:04:05' "$LANE_TGT/slim/x86_64-unknown-linux-gnu/server-dev/deps/libx.rlib"
"$LST" copy "$LANE_TGT/slim" "$LANE_TGT/slim-dst" 2>/dev/null || bad "slim copy failed"
[ -e "$LANE_TGT/slim-dst/x86_64-unknown-linux-gnu/server-dev/deps/libx.rlib" ] || bad "slim copy lost a dep"
[ -e "$LANE_TGT/slim-dst/x86_64-unknown-linux-gnu/server-dev/incremental" ] && bad "slim copy carried a <triple>/<profile>/incremental dir"
[ -e "$LANE_TGT/slim-dst/debug/incremental" ] && bad "slim copy carried a <profile>/incremental dir"
[ -e "$LANE_TGT/slim-dst/.tree-owner" ] && bad "slim copy carried .tree-owner"
[ "$(stat -c %Y "$LANE_TGT/slim-dst/x86_64-unknown-linux-gnu/server-dev/deps/libx.rlib")" = "$(stat -c %Y "$LANE_TGT/slim/x86_64-unknown-linux-gnu/server-dev/deps/libx.rlib")" ] || bad "slim copy changed an mtime (cargo freshness)"
[ "$(stat -c %i "$LANE_TGT/slim-dst/x86_64-unknown-linux-gnu/server-dev/deps/libx.rlib")" = "$(stat -c %i "$LANE_TGT/slim-dst/x86_64-unknown-linux-gnu/server-dev/deps/libx-link.rlib")" ] || bad "slim copy broke a hard link"
LANE_COPY_INCREMENTAL=1 "$LST" copy "$LANE_TGT/slim" "$LANE_TGT/slim-full" 2>/dev/null
[ -e "$LANE_TGT/slim-full/debug/incremental/big" ] || bad "LANE_COPY_INCREMENTAL=1 should keep the incremental dirs"
# the guard sizes the slim copy: a disk that fits it but not the full tree passes
export LANE_DISK_MARGIN_KB=0
full=$(du -sk "$LANE_TGT/slim" | cut -f1); slim=$(du -sk --exclude=incremental "$LANE_TGT/slim" | cut -f1)
[ "$slim" -lt "$full" ] || bad "fixture: the incremental dirs should add size"
export LANE_FREE_KB=$(( slim + slim / 5 + 8 ))
"$LST" check-disk "$LANE_TGT/slim" 2>/dev/null || bad "the guard counted the incremental dirs a slim copy leaves out"
LANE_COPY_INCREMENTAL=1 "$LST" check-disk "$LANE_TGT/slim" 2>/dev/null && bad "the full-copy guard ignored the incremental dirs"
unset LANE_FREE_KB LANE_DISK_MARGIN_KB

if [ "$fails" -ne 0 ]; then echo "check-lane-lock: FAIL -- $fails problem(s)" >&2; exit 1; fi
echo "check-lane-lock: OK -- 10 scenarios"
