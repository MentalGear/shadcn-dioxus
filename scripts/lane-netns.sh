#!/usr/bin/env bash
# Run a command in a PRIVATE network namespace with only loopback up (backlog row 221).
#
#   scripts/lane-netns.sh cmd [args...]
#
# Why: `dx build --ssg` (0.7.9) pre-renders by starting the built server on a FIXED port,
# 127.0.0.1:9999, and fetching every route from it. Two SSG pre-renders alive at once -- two queue
# slots, or a slot and an old-style `flock ssg.lock` caller -- share that port: the second dx
# talks to the FIRST one's server, finishes in 0.4 s with "SSG complete" and no error line, and
# writes no page (measured 2026-10-19: "95 of 96 route(s) missing" in the loser, the winner fine).
# Inside its own namespace each build owns 127.0.0.1:9999, so the collision cannot happen by
# construction. The build needs no network (the template's dependency cache is warm).
# Falls back to running the command directly (loudly) when the container refuses `unshare -n`.
set -euo pipefail
if unshare --net true 2>/dev/null; then
  exec unshare --net python3 -c '
import fcntl, os, socket, struct, sys
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
fcntl.ioctl(s, 0x8914, struct.pack("16sH14s", b"lo", 0x1 | 0x8 | 0x40, b""))  # SIOCSIFFLAGS: lo UP|LOOPBACK|RUNNING
os.execvp(sys.argv[1], sys.argv[1:])' "$@"
fi
echo "lane-netns: WARNING: unshare --net is not permitted here; running without a private loopback (concurrent SSG builds will collide on :9999)" >&2
exec "$@"
