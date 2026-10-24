#!/usr/bin/env bash
#
# check-js-bundle.sh
#
# `primitives/src/js/*.js` is generated from `primitives/src/ts/*.ts` and committed (dioxus-primitives has no build
# script), so `primitives/src/js/bundle.manifest`, written only by `scripts/build-js-bundle.sh`, records the sha256 of
# every `.ts` and `.js`: the gate fails on an edit of either side without a rebuild, an orphan, or a hand-edited
# `.js`. When the installed `bun` is the recorded version it also rebuilds and requires identical bytes, otherwise it
# reports the rebuild half as skipped. No network.
#
# `primitives/src/js/*.js` is GENERATED from `primitives/src/ts/*.ts` and committed, so dioxus-primitives needs no
# build script (it used to have `build.rs` + the `lazy-js-bundle` crate, which ran in every consumer's build;
# dev-docs/backlog.md row 217). Committing a generated file makes one class possible: the source and the artifact
# drifting apart -- a `.ts` edit without a rebuild (the next build ships the old behaviour), or a hand edit of the
# `.js` (the next rebuild erases it; `dev-docs/lifting-from-forks.md` section 2 lists a fork fix that did exactly
# that). This gate removes the class:
#   - `primitives/src/js/bundle.manifest` (written only by `scripts/build-js-bundle.sh`) records the sha256 of every
#     `src/ts/*.ts` and of the `src/js/*.js` built from it; any edit of either side without a rebuild is red;
#   - every `src/ts/<n>.ts` has a `src/js/<n>.js` and vice versa (no orphans, nothing outside the manifest);
#   - when `bun` is installed AND its version equals the one recorded in the manifest, the gate also rebuilds each
#     file into a temp dir and requires the bytes to be identical (the deterministic-rebuild half). A different
#     bun minifies differently (bun 1.4.2 vs the 2024 build of the original 1517-byte file), so a version mismatch
#     is reported as "rebuild check skipped", never as a failure, and the sha256 half still holds. No network.
#
# Usage: scripts/check-js-bundle.sh
# Exit status: 0 when clean.

set -uo pipefail
cd "$(dirname "$0")/.."

ts_dir=primitives/src/ts
js_dir=primitives/src/js
manifest=$js_dir/bundle.manifest
fail=0
err() { echo "check-js-bundle: $*" >&2; fail=1; }

[ -f "$manifest" ] || { echo "check-js-bundle: $manifest is missing; run scripts/build-js-bundle.sh" >&2; exit 1; }

recorded_bun="$(sed -n 's/^bun //p' "$manifest")"
declare -A want_ts want_js
while read -r kind sum name; do
  case "$kind" in
    ts) want_ts["$name"]="$sum" ;;
    js) want_js["$name"]="$sum" ;;
  esac
done < <(grep -E '^(ts|js) ' "$manifest")

shopt -s nullglob
declare -A seen_ts seen_js
for f in "$ts_dir"/*.ts; do
  n="$(basename "$f")"; seen_ts["$n"]=1
  [ -n "${want_ts[$n]:-}" ] || { err "$f is not in $manifest; run scripts/build-js-bundle.sh"; continue; }
  [ "$(sha256sum "$f" | cut -d' ' -f1)" = "${want_ts[$n]}" ] \
    || err "$f changed since $manifest was written: run scripts/build-js-bundle.sh and commit the .js and the manifest"
  j="$js_dir/$(basename "$n" .ts).js"
  [ -f "$j" ] || err "$j (built from $f) is missing; run scripts/build-js-bundle.sh"
done
for f in "$js_dir"/*.js; do
  n="$(basename "$f")"; seen_js["$n"]=1
  [ -f "$ts_dir/$(basename "$n" .js).ts" ] || { err "$f has no $ts_dir/$(basename "$n" .js).ts source (a generated file must come from one)"; continue; }
  [ -n "${want_js[$n]:-}" ] || { err "$f is not in $manifest; run scripts/build-js-bundle.sh"; continue; }
  [ "$(sha256sum "$f" | cut -d' ' -f1)" = "${want_js[$n]}" ] \
    || err "$f does not match $manifest: it was edited by hand or rebuilt without writing the manifest; edit $ts_dir/$(basename "$n" .js).ts and run scripts/build-js-bundle.sh"
done
for n in "${!want_ts[@]}"; do [ -n "${seen_ts[$n]:-}" ] || err "$manifest lists $ts_dir/$n, which no longer exists"; done
for n in "${!want_js[@]}"; do [ -n "${seen_js[$n]:-}" ] || err "$manifest lists $js_dir/$n, which no longer exists"; done

rebuild="skipped (no bun)"
if command -v bun >/dev/null; then
  have_bun="$(bun --version)"
  if [ "$have_bun" != "$recorded_bun" ]; then
    rebuild="skipped (bun $have_bun, manifest recorded bun $recorded_bun)"
  else
    tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
    rebuild="ok"
    for f in "$ts_dir"/*.ts; do
      n="$(basename "$f" .ts)"
      if bun build "$f" --outfile "$tmp/$n.js" --minify-whitespace --minify-syntax >/dev/null 2>&1; then
        cmp -s "$tmp/$n.js" "$js_dir/$n.js" \
          || { err "bun $have_bun rebuilds $f into different bytes than the committed $js_dir/$n.js; run scripts/build-js-bundle.sh"; rebuild="FAILED"; }
      else
        err "bun $have_bun could not build $f"; rebuild="FAILED"
      fi
    done
  fi
fi

[ "$fail" -ne 0 ] && exit 1
echo "check-js-bundle: OK -- ${#want_ts[@]} .ts source(s) match their committed .js and $manifest (deterministic rebuild: $rebuild)."
