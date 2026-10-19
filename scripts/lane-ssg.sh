#!/usr/bin/env bash
# Serialized debug SSG build for a lane worktree (one cargo/dx build at a time on this 4-core box).
#   scripts/lane-ssg.sh <worktree-abs-path> <lane-name> [port]   (LANE_TGT, default $T; see dev-docs/lane-rules.md)
# Under an exclusive flock: copies the warm, known-good N0 template to a fresh scratch target
# ($T/ssg-lane), builds there, copies public/ to $T/pub-<lane>, deletes the
# scratch target. (A target dir SHARED across worktrees failed 3/5 builds with "stylesheet ... not in
# dx's asset manifest", so every build now starts from the same clean tree.) Then (re)starts a static
# server on [port] (kills only the PID it recorded itself). Prints PUBLIC=<dir> and URL=<url>.
set -euo pipefail
T="${LANE_TGT:-/home/user/tgt}"
wt="${1:?worktree}"; lane="${2:?lane}"; port="${3:-}"
pub="$T/pub-$lane"; log="$T/ssg-$lane.log"; scratch=$T/ssg-lane
exec 9>$T/ssg.lock
echo "lane-ssg: waiting for build lock..." >&2
flock 9
echo "lane-ssg: building $wt (log $log)" >&2
rm -rf "$scratch"; cp -a $T/template "$scratch"
cd "$wt"
# Shared/copied targets: cargo gives a workspace crate the SAME unit hash in every worktree and judges
# freshness by mtime, so an older worktree file can be "fresh" against another tree's artifact (lane C
# measured builds containing another lane's code). Touch our sources so this tree always rebuilds.
find preview/src preview/assets preview/build.rs primitives/src dioxus-attributes/src -type f -exec touch {} +
if ! CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR="$scratch" CARGO_INCREMENTAL=0 scripts/build-ssg.sh debug >"$log" 2>&1; then
  rm -rf "$scratch"; tail -30 "$log" >&2; echo "lane-ssg: BUILD FAILED (full log $log)" >&2; exit 1
fi
rm -rf "$pub"; cp -a "$scratch/dx/preview/debug/web/public" "$pub"; rm -rf "$scratch"
flock -u 9
echo "PUBLIC=$pub"
if [ -n "$port" ]; then
  pidf="$T/serve-$lane.pid"
  [ -f "$pidf" ] && kill "$(cat "$pidf")" 2>/dev/null || true
  sleep 0.5
  # Lock fds closed: a long-lived child that inherits a locked file description holds the global
  # lock after its parent exits (an orphaned server once stalled every lane's build, row 221).
  nohup python3 -m http.server "$port" --bind 127.0.0.1 -d "$pub" >/dev/null 2>&1 9>&- 3>&- </dev/null &
  echo $! > "$pidf"
  echo "URL=http://127.0.0.1:$port"
fi
