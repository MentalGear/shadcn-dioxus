#!/usr/bin/env bash
# check-lane-target.sh
#
# The warm release trees keep their promise that a tree never holds the other's base-path setting, proven on tiny fake
# trees with no build: `lane-target.sh create <n> --release` copies `release-template` (no base path), a base-path
# copy only comes from `release` when its marker records that path, a tainted or sentinel tree is refused, `prune
# --apply` never removes either warm tree, `warm-release` skips a current tree, and `scripts/deploy-preview.sh`
# refuses the validation tree before it builds anything.
#
# A cold release build is 22-50 min; scripts/lane-target.sh keeps two warm release trees so a validation build is
# about 14 min, and the rule that makes that safe is "a tree never holds the other's base-path setting" (row 100):
#   - `create <n> --release` copies <root>/release-template (no base path), never <root>/release (the deploy's);
#   - `create <n> --release --base-path P` copies <root>/release, only if its marker records `--base-path P`;
#   - a no-base-path template that carries `.base-path-build` is refused, as is the sentinel tree for a base-path copy;
#   - `--release` and `--template` together are a usage error (the template is not a free choice);
#   - `prune --apply` never removes `release` or `release-template`, however idle;
#   - `warm-release` skips a tree whose recorded Cargo.lock hash is the current one (the refresh is for dependency changes);
#   - scripts/deploy-preview.sh defaults to <LANE_TGT>/release and refuses a tree with the `.no-base-path-only` sentinel
#     BEFORE it builds anything.
# Usage: scripts/check-lane-target.sh      (a few seconds, no network)
set -uo pipefail
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"
fail=0
ok()  { printf '  ok   %s\n' "$1"; }
bad() { printf '  FAIL %s\n' "$1" >&2; fail=1; }
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
R="$tmp/root"
mk() { # tree name, marker text (optional), sentinel (optional "s")
  local d="$R/$1"; mkdir -p "$d/dx/preview/release/web/public" "$d/wasm-release"
  echo "$1" > "$d/wasm-release/id"; echo '<html></html>' > "$d/dx/preview/release/web/public/index.html"
  [ -z "${2:-}" ] || echo "$2" > "$d/.base-path-build"
  [ -z "${3:-}" ] || : > "$d/.no-base-path-only"
}
mk template
mk release-template "" s
mk release "dx build ... --base-path shadcn-dioxus (2026-10-20T09:34:45Z)"
t() { scripts/lane-target.sh "$@" --root "$R"; }

out="$(t create a --release 2>/dev/null)"; rc=$?
if [ $rc = 0 ] && [ "$(cat "$R/a/wasm-release/id" 2>/dev/null)" = release-template ] && [[ "$out" == *"export CARGO_TARGET_DIR=$R/a"* ]]; then ok "no base path -> release-template"; else bad "no base path -> release-template (rc=$rc)"; fi
[ ! -e "$R/a/dx/preview/release/web/public" ] && [ ! -e "$R/a/.base-path-build" ] && ok "the copy has no stale public/ and no base-path marker" || bad "the copy has no stale public/ and no marker"

out="$(t create b --release --base-path shadcn-dioxus 2>/dev/null)"; rc=$?
if [ $rc = 0 ] && [ "$(cat "$R/b/wasm-release/id" 2>/dev/null)" = release ] && [ -e "$R/b/.base-path-build" ]; then ok "base path -> release (marker kept)"; else bad "base path -> release (rc=$rc)"; fi

if t create c --release --base-path other >/dev/null 2>&1; then bad "a different base path must be refused"; else ok "a base path the deploy tree does not record is refused"; fi
if t create d --release --template "$R/template" >/dev/null 2>&1; then bad "--release with --template must be a usage error"; else ok "--release with --template is refused"; fi
if t create e --base-path shadcn-dioxus >/dev/null 2>&1; then bad "--base-path without --release must be refused"; else ok "--base-path without --release is refused"; fi

echo "dx build ... --base-path shadcn-dioxus (x)" > "$R/release-template/.base-path-build"
if t create f --release >/dev/null 2>&1; then bad "a tainted no-base-path tree must be refused"; else ok "a no-base-path tree with .base-path-build is refused"; fi
rm "$R/release-template/.base-path-build"
cp "$R/release-template/.no-base-path-only" "$R/release/.no-base-path-only"
if t create g --release --base-path shadcn-dioxus >/dev/null 2>&1; then bad "the sentinel tree must not be copied for a base-path build"; else ok "the sentinel tree is refused for a base-path copy"; fi
rm "$R/release/.no-base-path-only"

# prune: everything old, only the lane copies may go
find "$R" -mindepth 1 -maxdepth 3 -exec touch -d '30 days ago' {} + 2>/dev/null
t prune --older-than 2 --apply >/dev/null 2>&1
if [ -d "$R/release" ] && [ -d "$R/release-template" ] && [ -d "$R/template" ] && [ ! -d "$R/a" ] && [ ! -d "$R/b" ]; then ok "prune removes idle lane copies, never the warm trees"; else bad "prune kept/removed the wrong dirs: $(ls "$R" | tr '\n' ' ')"; fi

# warm-release: a tree built from the current Cargo.lock is current -- skipped before it queues anything
mkdir -p "$R/release-template/dx"; sha1sum Cargo.lock | cut -c1-40 > "$R/release-template/.cargo-lock-sha"
out="$(t warm-release 2>&1)"; rc=$?
if [ $rc = 0 ] && [[ "$out" == *"is current"* ]]; then ok "warm-release skips a tree whose dependencies match Cargo.lock"; else bad "warm-release must skip a current tree (rc=$rc): $out"; fi

# deploy-preview.sh: refuses the sentinel tree before building, and defaults to <LANE_TGT>/release
out="$(CARGO_TARGET_DIR="$R/release-template" timeout 20 scripts/deploy-preview.sh 2>&1)"; rc=$?
if [ $rc = 2 ] && [[ "$out" == *"no-base-path validation tree"* ]]; then ok "deploy-preview refuses the no-base-path tree"; else bad "deploy-preview must refuse the sentinel tree (rc=$rc)"; fi
: > "$R/release/.no-base-path-only"
out="$(env -u CARGO_TARGET_DIR LANE_TGT="$R" timeout 20 scripts/deploy-preview.sh 2>&1)"; rc=$?
if [ $rc = 2 ] && [[ "$out" == *"$R/release"* ]]; then ok "deploy-preview defaults to <LANE_TGT>/release"; else bad "deploy-preview default target is not <LANE_TGT>/release (rc=$rc)"; fi

if [ "$fail" = 0 ]; then echo "check-lane-target: OK"; else echo "check-lane-target: FAILED" >&2; fi
exit "$fail"
