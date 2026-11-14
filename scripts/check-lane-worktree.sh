#!/usr/bin/env bash
# `scripts/lane-worktree.sh add|remove` keeps its promises, proven in a throwaway repo with no build: a new lane worktree is on
# its branch with the main checkout's `node_modules` linked into `preview/`, `playwright/` and `scripts/ts-tools/` (a directory
# that the start point lacks gets no link and no error), `git status` there stays clean (the links are ignored), an existing path
# is refused and left untouched, an existing branch is checked out as it is and refuses a start point, a worktree created from
# inside another lane worktree links the main checkout's directories, and `remove` refuses a dirty worktree, removes a clean one
# and keeps its branch and the shared `node_modules`.
#
# A fresh `git worktree add` has no `node_modules`, so a lane's first Playwright or TypeScript-tool command failed until someone
# linked them by hand; scripts/lib/pinned-worktree.sh holds the one link step (scripts/gate-at.sh's detached worktrees use it
# too) and dev-docs/lane-rules.md names the command.
#
# Usage: scripts/check-lane-worktree.sh      Exit status: 0 = OK; 1 = a promise broke.
set -uo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "check-lane-worktree: FAIL -- $*"; exit 1; }
command -v git >/dev/null || fail "git is required"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
r="$tmp/main"
git init -q "$r" || fail "git init failed"
git -C "$r" config user.email gate@example.invalid
git -C "$r" config user.name gate
mkdir -p "$r/scripts/lib" "$r/preview" "$r/playwright"
cp "$repo_root/scripts/lane-worktree.sh" "$r/scripts/"
cp "$repo_root/scripts/lib/pinned-worktree.sh" "$r/scripts/lib/"
cp "$repo_root/.gitignore" "$r/"
echo x > "$r/preview/p.txt"
git -C "$r" add -A && git -C "$r" commit -qm old || fail "old commit failed"
old="$(git -C "$r" rev-parse HEAD)"
# the newer commit adds playwright/ and scripts/ts-tools/; the old one has no playwright/ and no ts-tools/
mkdir -p "$r/playwright" "$r/scripts/ts-tools"
echo x > "$r/playwright/s.txt"; echo x > "$r/scripts/ts-tools/t.txt"
git -C "$r" add -A && git -C "$r" commit -qm new || fail "new commit failed"
# the shared (untracked, ignored) node_modules of the main checkout: preview and playwright have one, ts-tools does not
mkdir -p "$r/preview/node_modules/pkg" "$r/playwright/node_modules/pkg"
echo marker > "$r/preview/node_modules/pkg/m"
git -C "$r" status --porcelain | grep -q . && fail "the scratch main checkout is not clean before the test"
lw="$r/scripts/lane-worktree.sh"
wts="$tmp/wts"; mkdir -p "$wts"

# 1. add: on its branch, linked where the main checkout has node_modules and the start point has the directory
out="$("$lw" add lane/one "$wts/one" 2>&1)" || { echo "$out"; fail "add failed"; }
[ "$(git -C "$wts/one" rev-parse --abbrev-ref HEAD)" = "lane/one" ] || fail "the worktree is not on lane/one"
[ "$(cat "$wts/one/preview/node_modules/pkg/m")" = marker ] || fail "preview/node_modules is not linked to the main checkout's"
[ -L "$wts/one/playwright/node_modules" ] || fail "playwright/node_modules is not linked"
[ ! -e "$wts/one/scripts/ts-tools/node_modules" ] || fail "scripts/ts-tools got a link although the main checkout has no node_modules there"
[ -z "$(git -C "$wts/one" status --porcelain)" ] || fail "git status in the new worktree is not clean: $(git -C "$wts/one" status --porcelain)"
git -C "$wts/one" check-ignore -q preview/node_modules || fail "the link is not ignored by .gitignore"

# 2. an existing path is refused and left untouched
mkdir -p "$wts/taken"; echo keep > "$wts/taken/f"
"$lw" add lane/two "$wts/taken" >/dev/null 2>&1 && fail "add overwrote an existing directory"
[ "$(cat "$wts/taken/f")" = keep ] || fail "a refused add changed the existing directory"
git -C "$r" show-ref --verify --quiet refs/heads/lane/two && fail "a refused add left a branch behind"
"$lw" add lane/two "$wts/one" >/dev/null 2>&1 && fail "add overwrote an existing worktree"

# 3. a start point without a directory: no link, no error (the old commit has no playwright/); a new branch starts there
out="$("$lw" add lane/old "$wts/old" "$old" 2>&1)" || { echo "$out"; fail "add at an older start point failed"; }
[ "$(git -C "$wts/old" rev-parse HEAD)" = "$old" ] || fail "the worktree is not at the start point"
[ ! -e "$wts/old/playwright" ] || fail "the old start point has a playwright directory"
[ -L "$wts/old/preview/node_modules" ] || fail "preview/node_modules is not linked at the old start point"
[ -z "$(git -C "$wts/old" status --porcelain)" ] || fail "git status at the old start point is not clean"

# 4. an existing branch is checked out as it is, and refuses a start point; a branch checked out elsewhere is refused
git -C "$r" branch lane/pre "$old"
"$lw" add lane/pre "$wts/pre" HEAD >/dev/null 2>&1 && fail "add accepted a start point for an existing branch"
"$lw" add lane/pre "$wts/pre" >/dev/null 2>&1 || fail "add of an existing branch failed"
[ "$(git -C "$wts/pre" rev-parse HEAD)" = "$old" ] || fail "the existing branch was moved"
"$lw" add lane/pre "$wts/pre2" >/dev/null 2>&1 && fail "add checked out a branch that is already checked out"

# 5. from inside a lane worktree the links still come from the main checkout, and a relative path is resolved against the cwd
( cd "$wts" && "$wts/one/scripts/lane-worktree.sh" add lane/three three >/dev/null 2>&1 ) || fail "add from inside a lane worktree failed"
[ "$(cat "$wts/three/preview/node_modules/pkg/m")" = marker ] || fail "the link made from a lane worktree does not reach the main checkout's node_modules"
case "$(readlink "$wts/three/preview/node_modules")" in "$r"/*) ;; *) fail "the link points at '$(readlink "$wts/three/preview/node_modules")', not the main checkout" ;; esac

# 6. remove: a dirty worktree is refused, a clean one is removed, its branch and the shared node_modules stay
echo dirt > "$wts/three/untracked.txt"
"$lw" remove "$wts/three" >/dev/null 2>&1 && fail "remove deleted a worktree with an untracked file"
[ -d "$wts/three" ] || fail "a refused remove still deleted the worktree"
rm "$wts/three/untracked.txt"
out="$("$lw" remove "$wts/three" 2>&1)" || { echo "$out"; fail "remove of a clean worktree failed"; }
[ ! -e "$wts/three" ] || fail "remove left the directory"
git -C "$r" show-ref --verify --quiet refs/heads/lane/three || fail "remove deleted the branch"
[ "$(cat "$r/preview/node_modules/pkg/m")" = marker ] || fail "remove touched the main checkout's node_modules"
"$lw" remove "$r" >/dev/null 2>&1 && fail "remove accepted the main checkout"
"$lw" remove "$wts/nowhere" >/dev/null 2>&1 && fail "remove accepted a path that is no worktree"
"$lw" bogus >/dev/null 2>&1 && fail "an unknown command exited 0"

echo "check-lane-worktree: OK -- add links node_modules (none where absent), is clean to git, refuses an existing path, handles old start points, existing branches and calls from a lane worktree; remove refuses dirt and keeps the branch"
