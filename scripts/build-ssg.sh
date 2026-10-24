#!/usr/bin/env bash
# The ONE way to run the preview's `dx build --ssg` (backlog row 116; rows 98/100).
# Used by scripts/deploy-preview.sh and by every verification lane.
#
#   CARGO_TARGET_DIR=/abs/isolated/dir scripts/build-ssg.sh [debug|release] [--base-path P] [--no-css-bundle]
#
# Default profile: debug (the cheap one; deploy-preview.sh passes `release --base-path
# shadcn-dioxus`). On success prints, as its last three lines,
#   build-ssg: OK -- N routes
#   PUBLIC_DIR=<$CARGO_TARGET_DIR/dx/preview/<profile>/web/public>
#   STALE_HTML=0
# and on ANY failure exits non-zero with `build-ssg: FAIL -- <reason>` as the last stderr line.
# "Exit 0" therefore means the site is complete, by construction (guard 4), not "dx exited 0":
# dx 0.7.9 logs `ERROR ssg error: reqwest::Error ... IncompleteMessage` for a route it failed to
# render, prints `SSG complete` and exits 0 with only some pages written.
#
# Guards -- each one closes a failure class by construction instead of relying on the caller:
#  1. CARGO_TARGET_DIR must be set and ABSOLUTE. dx 0.7.9's `--ssg` pass chdirs before exec'ing
#     the still-relative server binary and dies with an opaque "No such file or directory
#     (os error 2)" (row 98); and an unset value means the shared repo `target/` (row 100).
#  2. A no-base-path build refuses a tree carrying the `.base-path-build` marker: such a tree
#     keeps emitting `/<base-path>/` URLs even after `rm -rf public` (row 100 -- the
#     tree is the problem, not its output). The marker is written HERE, before any
#     `--base-path` build; scripts/lane-target.sh refuses marked trees as lane templates.
#     The vice-versa direction (base-path build into an unmarked tree) is NOT refused: no
#     evidence of harm was ever recorded, the marker cannot detect it, and refusing would block
#     the normal first deploy into a warm tree. What is real there is a wrong *output*, so both
#     directions are checked on the output instead (step 5).
#  3. The profile's `dx/preview/<profile>/web/public` is wiped before building. dx 0.7.9 `--ssg`
#     only rewrites `public/index.html` when `public/` already exists; every other pre-rendered
#     route keeps the previous build's HTML (stale theme/main CSS hashes -- false "border-box
#     regressions"). It also never cleans old content-hashed wasm/js out of `public/`.
#  4. Postcondition (scripts/ssg-verify.sh, run before AND after the CSS bundle; its --self-test
#     covers the logic against a fake public/): dx's output is teed to a log whose failure
#     lines are grepped (`ssg error:`), and EVERY route dx was asked to pre-render (its own
#     `Rendering <route> for SSG` lines, cross-checked against the component CATALOG so a
#     truncated route list cannot verify itself) must have an index.html that is fresh (newer than a
#     marker taken just before dx starts), >= 4000 bytes and complete; no leftover index.html may exist.
#     Every step's exit status is propagated (`set -euo pipefail`; dx's own status is read from
#     PIPESTATUS because it is piped through tee).
#  4a. Retry, ONCE, loudly: a transport-level render failure (`reqwest ... IncompleteMessage`/connection
#     reset/refused/timeout in the dx log, seen on a cold first render of ~90 routes at once) or the
#     bundler's "not in dx's asset manifest" (a source file edited between the server compile and dx's
#     asset bundling, so the pages link a hash the manifest lacks) rebuilds from a wiped public/ and
#     manifest; guard 4 then runs again on the second build. Anything else fails at once. A second
#     failure fails the build.
#  0. Sources are touched before the build (BUILD_SSG_TOUCH=always|auto|never, default auto): see "Mtime discipline" below.
#  5. Output matches the requested base path: with `--base-path P` the root page must reference
#     `/P/...`; without it NO page may reference `/shadcn-dioxus/...` or the pre-rename `/dioxus-components/...` (same pattern as
#     scripts/lane-target.sh's contamination grep).
#  4b. Every page's <head> carries ONE render-blocking bundle of every component stylesheet
#     (scripts/ssg-css-bundle.mjs, run between guards 4 and 5 so guard 5 checks the final output). The
#     per-component `document::Link`s only reach the head of the route that SSR-renders them; on a
#     client-side navigation or a popover's first open they are inserted after first paint and are not
#     render-blocking, so the new content paints with UA defaults (white inputs, no radius) and then
#     reflows. `--no-css-bundle` builds the unbundled shape, for before/after measurements only -- never deploy it.
set -Eeuo pipefail

failed_line=""
fail_reported=0
trap 'failed_line=$LINENO' ERR
# fail "<one-line reason>" [detail line ...]: details first, the FAIL line LAST, then exit 1.
fail() {
  local reason="$1"; shift
  local d
  for d in "$@"; do echo "$d" >&2; done
  echo "build-ssg: FAIL -- $reason" >&2
  fail_reported=1
  exit 1
}
cleanup() {
  local rc=$?
  rm -f "${build_marker:-}" "${bundle_log:-}" "${reason_file:-}"
  # a `set -e` abort from a command nobody wrapped in fail() must still say FAIL
  if [ "$rc" -ne 0 ] && [ "$fail_reported" = 0 ]; then
    echo "build-ssg: FAIL -- command failed (exit $rc) at ${BASH_SOURCE[0]}:${failed_line:-?}" >&2
  fi
}
trap cleanup EXIT

profile=debug
base_path=""
css_bundle=1
while [ $# -gt 0 ]; do
  case "$1" in
    debug|release) profile="$1"; shift ;;
    --no-css-bundle) css_bundle=0; shift ;;
    --base-path) base_path="${2:?--base-path needs a value}"; shift 2 ;;
    --base-path=*) base_path="${1#--base-path=}"; shift ;;
    *) echo "usage: CARGO_TARGET_DIR=/abs/dir $0 [debug|release] [--base-path P] [--no-css-bundle]" >&2; fail_reported=1; exit 2 ;;
  esac
done
base_path="${base_path#/}"; base_path="${base_path%/}"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# --- Guard 1: absolute, explicit CARGO_TARGET_DIR ---------------------------------------
if [ -z "${CARGO_TARGET_DIR:-}" ]; then
  fail "CARGO_TARGET_DIR is not set" \
    "error: CARGO_TARGET_DIR is not set. Every SSG build needs an explicit, isolated, ABSOLUTE" \
    "       target dir (dev-docs/backlog.md rows 98/100; e.g. eval \"\$(scripts/lane-target.sh create <name>)\")."
fi
target_dir="${CARGO_TARGET_DIR%/}"
case "$target_dir" in
  /*) ;;
  *)
    fail "CARGO_TARGET_DIR must be an absolute path, got: '$target_dir'" \
      "error: CARGO_TARGET_DIR must be an absolute path, got: '$target_dir'" \
      "       (dx build --ssg breaks on relative target dirs -- dev-docs/backlog.md row 98;" \
      "        e.g. CARGO_TARGET_DIR=/home/user/tgt/deploy)."
    ;;
esac
export CARGO_TARGET_DIR="$target_dir"
# The target root must EXIST before dx starts: the dx log is `tee`d into it, and tee opens its file at once,
# racing dx/cargo, which create a cold root a moment later. Losing that race leaves no log, and guard 4 then
# fails a complete cold build with "dx log .../build-ssg.dx.log not found" (row 216; reproducible with
# a stub dx that creates the root after 0.5 s). It is intermittent, so a warm rerun passes.
mkdir -p "$target_dir"
marker_file="$target_dir/.base-path-build"
public_dir="$target_dir/dx/preview/$profile/web/public"
manifest="$(dirname "$public_dir")/.manifest.json"
dx_log="$target_dir/build-ssg.dx.log"

# --- Guard 2: no-base-path build must not reuse a base-path-tainted tree ----------------
if [ -z "$base_path" ] && [ -e "$marker_file" ]; then
  fail "$target_dir carries $marker_file: use a fresh tree" \
    "error: $target_dir carries $marker_file ($(head -n1 "$marker_file"))." \
    "       A no-base-path build in a tree that ever held a --base-path build emits" \
    "       base-path (/shadcn-dioxus/) URLs everywhere, and wiping public/ does not fix it" \
    "       (dev-docs/backlog.md row 100). Use a fresh tree: eval \"\$(scripts/lane-target.sh create <name>)\"."
fi

echo "==> Target dir: $target_dir (public dir: $public_dir)"

if [ -n "$base_path" ]; then
  # Tag the tree BEFORE building (even a failed base-path build taints the cache). Row 100.
  mkdir -p "$target_dir"
  echo "dx build ... --base-path $base_path ($(date -u +%FT%TZ))" > "$marker_file"
fi

# Freshness marker for guard 4: an existing dir/200 does not prove it is THIS build's output
# (same discipline as dev-docs/dx-serve-hot-reload.md). Re-touched at the start of every attempt.
build_marker="$(mktemp)"
bundle_log="$(mktemp)"
reason_file="$(mktemp)"

dx_args=(build --platform web --ssg --features fullstack --force-sequential=true)
[ "$profile" = release ] && dx_args+=(--release)
[ -n "$base_path" ] && dx_args+=(--base-path "$base_path")

verify_args=("$public_dir" "$dx_log" "$build_marker" --catalog "$repo_root/preview/src/components/mod.rs")
[ -n "$base_path" ] && verify_args+=(--base-path "$base_path")

# The dx log's transport-level render failures that a rebuild can plausibly cure (guard 4a). Every
# `ssg error:` line must be one of these for the failure to be retryable; a server-side 500, a
# panic or anything unrecognised is deterministic and fails at once.
transient_ssg_errors() { # $1 = dx log; succeeds iff it has ssg errors and ALL are transport-level
  local all transient
  all="$(sed -e $'s/\x1b\\[[0-9;]*[A-Za-z]//g' "$1" | grep -c 'ssg error:' || true)"
  transient="$(sed -e $'s/\x1b\\[[0-9;]*[A-Za-z]//g' "$1" | grep 'ssg error:' | grep -cE 'IncompleteMessage|ConnectionReset|ConnectionRefused|connection (closed|reset|refused)|timed out|BrokenPipe' || true)"
  [ "$all" -gt 0 ] && [ "$all" = "$transient" ]
}

# build_once: one wipe + dx + postcondition + bundle + postcondition. Returns 0 on success.
# On failure sets fail_reason and retryable (1 = guard 4a applies); detail has already been printed.
fail_reason="" retryable=0 route_count=""
build_once() {
  fail_reason="" retryable=0

  # --- Guard 3: wipe this profile's public/ AND dx's asset manifest ------------------------
  # public/: dx 0.7.9 `--ssg` only rewrites public/index.html when public/ already exists (stale
  # theme/main CSS hashes on every other route) and never cleans old content-hashed wasm/js out of it.
  # manifest: the sibling `.manifest.json` that guard 4b maps stylesheets through; one left over from
  # an earlier build can only be wrong, so dx must regenerate it or its absence fails the build.
  rm -rf "${public_dir:?}" "${manifest:?}" "${dx_log:?}"
  touch "$build_marker"

  # `--force-sequential=true` is NOT optional: it avoids the dx SSG server/client-build race that
  # ships dead, never-hydrating pages. Full analysis: the comment above the "Verifying every
  # route's build output includes its hydration bootstrap script" step in scripts/deploy-preview.sh.
  echo "==> Building preview ($profile, ssg, fullstack${base_path:+, base-path $base_path}) ... (dx log: $dx_log)"
  local dx_rc=0
  set +e
  # BUILD_SSG_NETNS=1 (scripts/lane-ssg.sh): dx pre-renders through a server on the fixed port 9999, so two
  # concurrent builds collide silently; a private loopback per build removes that (scripts/lane-netns.sh).
  local netns=(); [ "${BUILD_SSG_NETNS:-0}" = 1 ] && netns=("$repo_root/scripts/lane-netns.sh")
  ( cd "$repo_root/preview" && exec "${netns[@]}" dx "${dx_args[@]}" ) 2>&1 | tee "$dx_log"
  dx_rc=${PIPESTATUS[0]}
  set -e
  if [ "$dx_rc" -ne 0 ]; then
    fail_reason="dx build exited $dx_rc (log: $dx_log)"
    return 1
  fi

  # Test hook (scripts/ssg-verify.sh --self-test covers the logic; this proves the wiring end to end):
  # move one route's page away after dx, as a dropped render would. Never set by a real build.
  if [ -n "${BUILD_SSG_TEST_DROP_ROUTE:-}" ]; then
    echo "build-ssg: TEST HOOK: dropping ${BUILD_SSG_TEST_DROP_ROUTE} after dx" >&2
    mv "$public_dir/${BUILD_SSG_TEST_DROP_ROUTE#/}/index.html" "$public_dir/${BUILD_SSG_TEST_DROP_ROUTE#/}/index.html.dropped-by-test-hook"
  fi

  if [ ! -d "$public_dir" ]; then
    fail_reason="expected build output at $public_dir, not found (is dx writing to a different CARGO_TARGET_DIR than '$target_dir'?)"
    return 1
  fi

  # --- Guard 4: the build's output is complete, fresh and error-free -------------------------
  if ! SSG_VERIFY_REASON_FILE="$reason_file" "$repo_root/scripts/ssg-verify.sh" "${verify_args[@]}"; then
    fail_reason="$(head -n1 "$reason_file" 2>/dev/null || true)"
    [ -n "$fail_reason" ] || fail_reason="scripts/ssg-verify.sh failed"
    if transient_ssg_errors "$dx_log"; then retryable=1; fi
    return 1
  fi

  # --- Guard 4b: one render-blocking stylesheet bundle in every page's <head> ---------------
  # (before guard 5, so guard 5 judges the output that is actually shipped). The script refuses a tree that
  # is already bundled, so a re-run of this script (which wipes public/ first anyway) cannot stack bundles.
  if [ "$css_bundle" = 1 ]; then
    if ! command -v node >/dev/null 2>&1; then
      fail_reason="node is required for scripts/ssg-css-bundle.mjs (guard 4b); pass --no-css-bundle only for a measurement build"
      return 1
    fi
    if [ ! -f "$manifest" ]; then
      fail_reason="dx asset manifest not found at $manifest (guard 4b maps each page's sheets to their sources with it)"
      return 1
    fi
    local bundle_rc=0
    set +e
    node "$repo_root/scripts/ssg-css-bundle.mjs" "$public_dir" --manifest "$manifest" 2>&1 | tee "$bundle_log"
    bundle_rc=${PIPESTATUS[0]}
    set -e
    if [ "$bundle_rc" -ne 0 ]; then
      fail_reason="scripts/ssg-css-bundle.mjs failed (exit $bundle_rc): $(grep -m1 '^ssg-css-bundle:' "$bundle_log" || echo 'no message')"
      # A page links a hash the manifest lacks = a stylesheet changed between the server's compile and
      # dx's asset bundling. Name what changed since this attempt started (usually another lane's edit).
      if grep -q "not in dx's asset manifest" "$bundle_log"; then
        retryable=1
        local drifted
        drifted="$(find "$repo_root/preview/src" "$repo_root/preview/assets" "$repo_root/primitives/src" -type f -newer "$build_marker" 2>/dev/null | head -n 5 || true)"
        if [ -n "$drifted" ]; then
          echo "build-ssg: sources changed during this build (likely cause of the stale manifest):" >&2
          printf '  - %s\n' $drifted >&2
        fi
      fi
      return 1
    fi
    # Every page, not a sample: a page without the bundle is a page that can flash.
    local unbundled
    unbundled="$(grep -rL --include=index.html 'data-dx-css-bundle' "$public_dir" | head -n 3 || true)"
    if [ -n "$unbundled" ]; then
      echo "error: page(s) without the CSS bundle after guard 4b:" >&2
      printf '  - %s\n' $unbundled >&2
      fail_reason="page(s) without the CSS bundle after guard 4b"
      return 1
    fi
  else
    echo "warning: --no-css-bundle: this output has the per-route stylesheet shape (flash on client-side navigation); do not deploy it." >&2
  fi

  # The bundle rewrites every page in place: the tree that ships must pass the postcondition too.
  # Its route count is the one reported on success.
  if ! SSG_VERIFY_REASON_FILE="$reason_file" "$repo_root/scripts/ssg-verify.sh" "${verify_args[@]}" > "$bundle_log"; then
    fail_reason="after the CSS bundle: $(head -n1 "$reason_file" 2>/dev/null || echo 'ssg-verify failed')"
    return 1
  fi
  route_count="$(sed -nE 's/^ssg-verify: OK -- ([0-9]+) routes.*/\1/p' "$bundle_log")"
  return 0
}

# --- Mtime discipline (row 216): cargo judges a workspace crate's freshness by mtime and gives it the same unit
# hash in every worktree, so a target another tree built in can look "fresh" against OLDER files here and
# the build would ship that tree's code. BUILD_SSG_TOUCH=auto (default) touches the sources unless this
# target's `.tree-owner` stamp already names this worktree; `always` forces it; `never` is for a caller that
# already did (scripts/lane-ssg.sh).
case "${BUILD_SSG_TOUCH:-auto}" in
  auto)   (cd "$repo_root" && scripts/lane-touch.sh "$target_dir") ;;
  always) (cd "$repo_root" && scripts/lane-touch.sh "$target_dir" --force) ;;
  never)  ;;
  *) fail "BUILD_SSG_TOUCH must be always, auto or never, got '${BUILD_SSG_TOUCH}'" ;;
esac

for attempt in 1 2; do
  if build_once; then break; fi
  if [ "$retryable" = 1 ] && [ "$attempt" = 1 ]; then
    {
      echo "build-ssg: ################################################################"
      echo "build-ssg: ATTEMPT 1 FAILED (transient): $fail_reason"
      echo "build-ssg: RETRYING ONCE from a wiped public/ and manifest; the postcondition runs again."
      echo "build-ssg: ################################################################"
    } >&2
    continue
  fi
  if [ "$attempt" = 2 ]; then fail_reason="$fail_reason (after the one allowed retry)"; fi
  fail "$fail_reason"
done

# --- Guard 5: output matches the requested base path -------------------------------------
if [ -n "$base_path" ]; then
  if ! grep -qE "[\"'(]/$base_path/" "$public_dir/index.html"; then
    fail "built with --base-path $base_path but $public_dir/index.html references no /$base_path/ URL" \
      "error: the output would 404 on its host (stale cached artifacts in $target_dir?)."
  fi
else
  if hit="$(grep -rlE --include='*.html' "[\"'(]/(shadcn-dioxus|dioxus-components)/" "$public_dir" | head -n 3)" && [ -n "$hit" ]; then
    echo "error: no --base-path requested but the output references /shadcn-dioxus/ (or legacy /dioxus-components/) URLs (row 100):" >&2
    printf '  - %s\n' $hit >&2
    echo "       This tree carries base-path build artifacts; use a fresh one (scripts/lane-target.sh create)." >&2
    fail "no --base-path requested but the output references base-path URLs (row 100)"
  fi
fi

# A debug build says so (row 339): playwright/base-url.ts's buildProfile() reads it so the timing specs skip on a
# lane build whose path names no profile. Never on release: that public dir is what deploys.
if [ "$profile" = debug ]; then echo debug > "$public_dir/build-profile.txt"; else rm -f "$public_dir/build-profile.txt"; fi

echo "build-ssg: OK -- ${route_count:-?} routes"
echo "PUBLIC_DIR=$public_dir"
echo "STALE_HTML=0"
