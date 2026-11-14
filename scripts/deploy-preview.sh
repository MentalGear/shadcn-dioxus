#!/usr/bin/env bash
# Builds the preview app (`preview/`) and publishes it into /docs for GitHub
# Pages, which is configured (repo Settings -> Pages) to serve straight
# from the `main` branch's /docs folder -- no CI build step involved.
#
# Usage: scripts/deploy-preview.sh               (the release suite, then the deploy build)
#        scripts/deploy-preview.sh --suite-only  (only the release suite: run it ahead of a deploy, e.g. nightly)
# Then: git add docs && git commit -m "..." && git push (to main).
#
# THE FULL PLAYWRIGHT SUITE RUNS HERE, ONCE PER DEPLOY (backlog row 322). A merge batch runs the gates and the specs its
# diff selects (dev-docs/lane-rules.md, "Merge queue"); the whole suite (~66-82 min on release) is what stands between
# main and the published site. Before building, the deploy needs a green release-suite run of the EXACT tree it
# deploys: a stamp `$LANE_TGT/release-suite/<tree sha>.pass` (`git rev-parse HEAD^{tree}`, clean worktree). With no
# stamp it makes one: a no-base-path RELEASE build of this tree in a copy of the warm `release-template` (the suite
# cannot run on the base-path build, row 100), served by serve-spa.py, `playwright/tools/run-suite.sh` on it, and the
# stamp only when every chunk reported and no test failed (flaky = passed on retry is allowed and listed). A red suite
# stops the deploy with the failing titles. `DEPLOY_SUITE=skip:<reason>` deploys without it and says so; it is for an
# emergency fix only, and the reason is printed.
# REUSE ACROSS SPEC-ONLY CHANGES (backlog row 513). A red run followed by a fix that touches only Playwright files leaves the
# site byte-identical, so the suite does not run again in full. Every spec file that ran writes a record, green only when every
# test of the file passed, under (site key, spec key) in `$LANE_TGT/release-suite/records` (playwright/tools/suite-key.mjs:
# the site key hashes every input of the release build, the spec key the file, what it imports, the config, the data and the
# environment it reads; the record also names the hash of the built site). A tree is accepted, without a build, when every spec
# file of it has a green record for its site and all the records name one built site; otherwise the files without a record
# run, on the kept build of that site (`$LANE_TGT/release-suite/site/<site key>/public`, ~55 MB, the two newest are kept) or on a
# fresh one. The totals it prints are for the whole suite (reused plus run) and say how many files were reused. Any doubt (an
# untracked file, a key that cannot be derived, a build that hashes differently) runs the whole suite.
# A record is green by proof (the tests of the report equal an independent `--list`, each expected or flaky; a file with a skipped,
# interrupted or unrun test gets none and runs every time), and the worktree must still be the planned tree when the run ends.
# `DEPLOY_SUITE_REUSE=off` turns the records off.
#   DEPLOY_SUITE_WORKERS (default 3: the main loop's full-run setting), DEPLOY_SUITE_CHUNKS (default 6),
#   DEPLOY_SUITE_PORT (default 4830).
# The `@timing` cases (playwright/timing.ts) are not in the six chunks: a seventh pass runs them alone, one worker,
# once the machine is quiet (playwright/tools/timing-pass.sh), and a pass that never ran counts as red, unless the
# switch (playwright/tools/timing-switch.sh) is off: then the pass is deactivated (backlog row 494) and its missing
# report is not red.
# The deploy suite is the one place the narrowed browser sweeps run in full (row 329: the cargo `ssr_parity` test
# covers SSR-vs-first-render per batch): `FIRST_PAINT_ROUTES=all` (first-paint text and styles, every route) and
# `LS_STEP=32` (layout-stability, every width on every route).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$repo_root/playwright/tools/timing-switch.sh"
suite_only=0
case "${1:-}" in
  "") ;;
  --suite-only) suite_only=1 ;;
  *) echo "deploy-preview: unknown argument $1 (usage: scripts/deploy-preview.sh [--suite-only])" >&2; exit 2 ;;
esac

lane_tgt="${LANE_TGT:-/home/user/tgt}"
suite_dir="$lane_tgt/release-suite"

# Prints the whole-suite summary of a run-suite.sh output directory (the files that ran, from their chunk reports, plus the reused
# files, from their records, checked again) and exits non-zero unless every spec file of the tree is green, every chunk that had
# files wrote its JSON report with no run error, and no test failed (`unexpected`). The failing titles go to stderr; flaky
# (passed on retry) is allowed and listed. With a timing pass (playwright/tools/timing-switch.sh) a missing or erroring
# chunk-timing.json counts as a missing chunk; while it is off the line says so.
# $3 (the tree) is always given: the verdict is red when HEAD's tree is not it or a tracked file changed during the run.
# $4 (the record store) is given when records are in use.
suite_verdict() {
  local args=(verdict --out "$1" --chunks "$2" --tree "$3")
  [ -z "${4:-}" ] || args+=(--store "$4")
  ( cd "$repo_root/playwright" && node tools/suite-key.mjs "${args[@]}" )
}

# The build environment string both keys of playwright/tools/suite-key.mjs hash: the tool versions the release build runs
# with and the variables that steer cargo. A tool that does not answer makes the string unique, so nothing is reused.
suite_build_env() {
  local v
  v="$( cd "$repo_root" && { dx --version && rustc -Vv && cargo -V; } 2>&1 )" || v="unavailable-$$-$(date +%s%N)"
  printf '%s\n%s\n' "$v" "$(env | grep -E '^(RUSTFLAGS|RUSTDOCFLAGS|CARGO_ENCODED_RUSTFLAGS|CARGO_BUILD_TARGET|CARGO_PROFILE_)' | sort)"
}

# playwright/tools/suite-key.mjs with the environment the deploy's suite runs under (the variables the specs read).
suite_key() {
  ( cd "$repo_root/playwright" && PW_BUILD_PROFILE=release FIRST_PAINT_ROUTES=all LS_STEP=32 node tools/suite-key.mjs "$@" )
}

release_suite() {
  local tree head stamp
  head="$(git -C "$repo_root" rev-parse HEAD)"
  tree="$(git -C "$repo_root" rev-parse 'HEAD^{tree}')"
  stamp="$suite_dir/$tree.pass"
  case "${DEPLOY_SUITE:-run}" in
    run) ;;
    skip:?*) echo "deploy-preview: WARNING: release suite SKIPPED (${DEPLOY_SUITE#skip:}); $head is deployed without a full-suite run (row 322)" >&2; return 0 ;;
    *) echo "deploy-preview: DEPLOY_SUITE must be unset, 'run' or 'skip:<reason>' (got '${DEPLOY_SUITE}')" >&2; exit 2 ;;
  esac
  # The stamp names a commit's tree; uncommitted changes would be deployed without having been tested. /docs is the
  # deploy's own output and does not count.
  if [ -n "$(git -C "$repo_root" status --porcelain --untracked-files=no -- . ':(exclude)docs')" ]; then
    echo "deploy-preview: the worktree has uncommitted changes outside docs/; commit them first, the release suite (row 322) runs on a commit's tree" >&2
    exit 2
  fi
  if [ -f "$stamp" ]; then
    echo "==> Release suite: green for tree $tree already ($(head -1 "$stamp")); not run again."
    return 0
  fi
  local name="deploy-suite-$$" out="$suite_dir/run-$tree" port="${DEPLOY_SUITE_PORT:-4830}"
  local chunks="${DEPLOY_SUITE_CHUNKS:-6}" workers="${DEPLOY_SUITE_WORKERS:-3}"
  local tgt="" pub server_pid=""
  # Row 513: per-file records. Off on any doubt; the stamp above stays the whole-tree fast path.
  local store="$suite_dir/records" use_records=0 skey="" kept=""
  if [ "${DEPLOY_SUITE_REUSE:-on}" = on ]; then
    if [ -n "$(git -C "$repo_root" ls-files --others --exclude-standard -- . ':(exclude)playwright' ':(exclude)dev-docs' ':(exclude)docs' ':(exclude)primitives/ts-test')" ]; then
      echo "deploy-preview: untracked files outside the excluded paths: per-file reuse is off, the whole suite runs" >&2
    else
      SUITE_BUILD_ENV="$(suite_build_env)"; export SUITE_BUILD_ENV
      if skey="$(suite_key site-key --tree "$tree" --build-env "$SUITE_BUILD_ENV" | node -e 'process.stdout.write(JSON.parse(require("fs").readFileSync(0, "utf8")).key)')" && [ -n "$skey" ]; then
        use_records=1; mkdir -p "$suite_dir" "$store"
        if suite_key accept --store "$store" --tree "$tree" --build-env "$SUITE_BUILD_ENV" | tee "$suite_dir/accept-$tree.log" >&2; then
          local totals; totals="$(tail -1 "$suite_dir/accept-$tree.log")"
          printf '%s %s workers=%s %s (every spec file has a green record for site %s)\n' "$(date -u +%FT%TZ)" "$head" "$workers" "$totals" "${skey:0:12}" > "$stamp"
          echo "==> Release suite GREEN from per-file records, no build and no run: $totals (stamp $stamp)"
          return 0
        fi
      else
        echo "deploy-preview: no site key for tree $tree; per-file reuse is off, the whole suite runs" >&2
      fi
    fi
  fi
  echo "==> Release suite (row 322): no green run recorded for tree $tree ($head); building and running it."
  mkdir -p "$suite_dir"; rm -rf "$out"; mkdir -p "$out"
  trap '[ -n "${server_pid:-}" ] && kill "${server_pid}" 2>/dev/null' EXIT
  if [ "$use_records" = 1 ]; then kept="$suite_dir/site/$skey"; fi
  if [ -n "$kept" ] && [ -d "$kept/public" ] && [ "$(suite_key hash-dir "$kept/public")" = "$(cat "$kept/public.sha" 2>/dev/null)" ]; then
    pub="$kept/public"
    echo "==> Release suite: serving the kept build of site ${skey:0:12} ($kept); no build."
  else
    tgt="$(cd "$repo_root" && scripts/lane-target.sh create "$name" --release | sed -n 's/^export CARGO_TARGET_DIR=//p')"
    [ -n "$tgt" ] || { echo "deploy-preview: could not copy the warm release-template tree (scripts/lane-target.sh create $name --release)" >&2; exit 1; }
    # shellcheck disable=SC2064
    trap "[ -n \"\${server_pid:-}\" ] && kill \"\${server_pid}\" 2>/dev/null; rm -rf '$tgt'" EXIT
    ( cd "$repo_root" && CARGO_TARGET_DIR="$tgt" CARGO_INCREMENTAL=0 scripts/lane-lock.sh gate --label "deploy-suite:build" -- \
        scripts/build-ssg.sh release ) > "$out/build.log" 2>&1 \
      || { tail -30 "$out/build.log" >&2; echo "deploy-preview: the release-suite build failed (log $out/build.log)" >&2; exit 1; }
    pub="$tgt/dx/preview/release/web/public"
    "$repo_root/scripts/emit-404.sh" "$pub" >/dev/null
    if [ -n "$kept" ]; then
      # Keep the built site (the part of the build a second run needs; the 3.6 GB target is removed after the run): ~55 MB per site,
      # the two newest are kept.
      rm -rf "$kept"; mkdir -p "$kept"
      cp -a "$pub" "$kept/public.tmp" && suite_key hash-dir "$kept/public.tmp" > "$kept/public.sha" && mv "$kept/public.tmp" "$kept/public" \
        || { echo "deploy-preview: could not keep the built site; the next run builds again" >&2; rm -rf "$kept"; }
      # shellcheck disable=SC2012
      ls -1dt "$suite_dir"/site/*/ 2>/dev/null | tail -n +3 | xargs -r -d '\n' rm -rf
    fi
  fi
  # The port must be free before the server starts, and what answers afterwards must be OUR server on THIS site: a stale server
  # on the port would answer the readiness probe and the suite would test another site.
  if curl -s -o /dev/null --max-time 2 "http://127.0.0.1:$port/"; then
    echo "deploy-preview: port $port already answers (a stale server?); refusing to test whatever it serves. Stop it or set DEPLOY_SUITE_PORT." >&2
    exit 1
  fi
  python3 "$repo_root/scripts/serve-spa.py" "$pub" "$port" >/dev/null 2>&1 9>&- 3>&- </dev/null &
  server_pid=$!
  for _ in $(seq 1 50); do curl -fsS -o /dev/null "http://127.0.0.1:$port/" && break; kill -0 "$server_pid" 2>/dev/null || break; sleep 0.2; done
  if ! kill -0 "$server_pid" 2>/dev/null || ! curl -fsS "http://127.0.0.1:$port/" | cmp -s - "$pub/index.html"; then
    echo "deploy-preview: the server on port $port is not this run's server for $pub (it died, or answers with other bytes than $pub/index.html); not running the suite" >&2
    exit 1
  fi
  local reuse_args=()
  if [ "$use_records" = 1 ]; then reuse_args=(--reuse "$store" --tree "$tree"); fi
  ( cd "$repo_root/playwright" && PLAYWRIGHT_BASE_URL="http://127.0.0.1:$port" PW_BUILD_PROFILE=release SSG_SITE_DIR="$pub" \
      FIRST_PAINT_ROUTES=all LS_STEP=32 \
      tools/run-suite.sh --out "$out" --chunks "$chunks" --workers "$workers" ${reuse_args[@]+"${reuse_args[@]}"} ) | tee "$out/run-suite.log" || true  # the verdict below decides
  kill "$server_pid" 2>/dev/null || true; server_pid=""
  [ -z "$tgt" ] || rm -rf "$tgt"; trap - EXIT
  local verdict
  if verdict="$(suite_verdict "$out" "$chunks" "$tree" "${reuse_args[1]:-}")"; then
    printf '%s %s workers=%s %s\n' "$(date -u +%FT%TZ)" "$head" "$workers" "$verdict" > "$stamp"
    echo "==> Release suite GREEN: $verdict (stamp $stamp; reports in $out)"
  else
    echo "deploy-preview: release suite RED for $head: $verdict -- not deploying (reports in $out; refresh nothing, fix and re-run)" >&2
    exit 1
  fi
}

cd "$repo_root/preview"

# Honor CARGO_TARGET_DIR (CLAUDE.md: every lane builds into an isolated, ABSOLUTE
# target dir; backlog rows 98/100/122). When it is unset the deploy builds in the WARM DEPLOY TREE,
# `${LANE_TGT:-/home/user/tgt}/release` (backlog row 324): the one release tree that is allowed to hold a
# `--base-path shadcn-dioxus` build, kept warm by every deploy, so a deploy rebuilds only the two workspace crates
# (~14 min) instead of the 700-crate dependency tree (~22-50 min). Without a /home/user/tgt it falls back to the
# workspace's own `target/`. The no-base-path validation tree (`release-template`, see
# scripts/lane-target.sh) is refused here: a base-path build would taint it for good (row 100).
# The build itself -- absolute-path / base-path-marker guards, wiping public/, the dx invocation, the freshness
# check -- lives in scripts/build-ssg.sh, the single implementation shared with verification lanes.
if [ -z "${CARGO_TARGET_DIR:-}" ] && [ -d "${LANE_TGT:-/home/user/tgt}" ]; then
  CARGO_TARGET_DIR="${LANE_TGT:-/home/user/tgt}/release"
fi
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo_root/target}"
target_dir="${CARGO_TARGET_DIR%/}"
if [ -e "$target_dir/.no-base-path-only" ]; then
  echo "deploy-preview: $target_dir is the no-base-path validation tree (sentinel .no-base-path-only); a --base-path build would taint it (row 100). Use the deploy tree: CARGO_TARGET_DIR=${LANE_TGT:-/home/user/tgt}/release" >&2
  exit 2
fi
mkdir -p "$target_dir"

# Row 322: the release suite gates the build (header). After the target checks above, so a wrong tree is refused
# before anything runs.
release_suite
if [ "$suite_only" = 1 ]; then exit 0; fi

# Same path build-ssg.sh derives for `release` (it also prints it as PUBLIC_DIR=).
public_dir="$target_dir/dx/preview/release/web/public"

# Wipes public/ first (dx's asset pipeline never cleans previous content-hashed
# wasm/js out of it), tags the tree `.base-path-build` (row 100) and verifies every
# page is fresh. `--force-sequential=true` is passed there, see the note below.
# It also owns the "is the site complete" postcondition (scripts/ssg-verify.sh: dx's own
# `ssg error:` lines, every requested route present/fresh/complete, every CATALOG component
# requested) and exits non-zero otherwise, so with `set -e` a deploy can never go on to
# refresh /docs from a build dx itself called a success while it dropped pages.
"$repo_root/scripts/build-ssg.sh" release --base-path shadcn-dioxus

# `dx build --ssg` has a race, independent of any source change, where its SSG render pass fires
# all routes' requests at the freshly-started fullstack server concurrently, and some of those
# requests can land before something the server needs to inject the hydration bootstrap
# `<script type="module">` is ready. The affected route's static HTML then has no script tag at
# all -- not a visible error, just a dead, never-hydrates page if it ships.
#
# The race is CPU contention, not cache state: by default `dx build --fullstack` compiles the
# server and client targets in parallel, and the client side's post-compile work (wasm-bindgen,
# wasm-opt -- both CPU-heavy) still competes for CPU while the SSG pass's concurrent requests hit
# the freshly-listening server. On this sandbox's 4 cores, `--force-sequential=true` above
# (server build, then client build, so nothing competes with the server during its SSG window)
# removes the race (row 116);
# dx's own `--help` default-enables it under `CI=1`. The verification below stays as a safety
# net regardless, on every route dx actually produced: a dead page must not ship even if this
# analysis is wrong for another environment.
echo "==> Verifying every route's build output includes its hydration bootstrap script ..."
missing_script=()
route_count=0
while IFS= read -r -d '' html_file; do
  route_count=$((route_count + 1))
  if ! grep -q '<script type="module"' "$html_file"; then
    missing_script+=("${html_file#"$public_dir"/}")
  fi
done < <(find "$public_dir" -name "index.html" -print0)

if [ "${#missing_script[@]}" -gt 0 ]; then
  echo "error: ${#missing_script[@]} of $route_count route(s) built WITHOUT their wasm" >&2
  echo "bootstrap <script type=\"module\"> tag -- they would ship as dead, never-" >&2
  echo "hydrating pages:" >&2
  printf '  - %s\n' "${missing_script[@]}" >&2
  echo "This looks like the dx/dioxus-fullstack SSG server/client-build race" >&2
  echo "described above -- but scripts/build-ssg.sh already passes --force-sequential=true" >&2
  echo "specifically to avoid it, so seeing this means that mitigation didn't hold" >&2
  echo "(different environment, dx version, core count, ...). A cold cache alone" >&2
  echo "does NOT fix this (see the comment above) -- diagnose" >&2
  echo "fresh from this build's own log rather than assume that remedy." >&2
  exit 1
fi
echo "==> OK: all $route_count route(s) have their bootstrap script."

# dev-docs/backlog.md row 46: a query-string route (`/component/?name=X&`) is not enumerable at
# build time, so it prerenders as the "Component not found" shell for every X; the demo routes are
# path segments (preview/src/site/demos.rs and block_demo.rs: ComponentDemoPath/ComponentBlockDemoPath). This check is the
# safety net for that class, on every route dx actually produced, the same belt-and-suspenders shape as the
# bootstrap-script check above. A well-formed build should never ship this
# marker at all: `ComponentDemoPath`'s own not-found branch only renders for
# a name absent from `components::DEMOS`, and `server_static_routes`
# (preview/src/main.rs) only ever asks dx to prerender names that ARE in
# that list; the legacy query-string routes (`ComponentDemo`/
# `ComponentBlockDemo`) render a differently-classed loading/redirect shell
# (`dx-component-demo-redirect`), not this marker.
echo "==> Verifying no route's build output is the \"Component not found\" shell ..."
not_found_routes=()
while IFS= read -r -d '' html_file; do
  if grep -q 'dx-component-demo-not-found' "$html_file"; then
    not_found_routes+=("${html_file#"$public_dir"/}")
  fi
done < <(find "$public_dir" -name "index.html" -print0)

if [ "${#not_found_routes[@]}" -gt 0 ]; then
  echo "error: ${#not_found_routes[@]} of $route_count route(s) built as the" >&2
  echo "\"Component not found\" shell (dx-component-demo-not-found) instead of" >&2
  echo "their real content -- this is dev-docs/backlog.md row 46's regression:" >&2
  printf '  - %s\n' "${not_found_routes[@]}" >&2
  echo "Check preview/src/main.rs's server_static_routes() actually lists this" >&2
  echo "route's name, and that the name matches a components::DEMOS entry." >&2
  exit 1
fi
echo "==> OK: no route shipped the not-found shell."

# dev-docs/backlog.md row 173: GitHub Pages answers every URL the SSG did not prerender
# with /404.html (status 404). Without one, an unknown URL gets GitHub's generic page and
# never reaches the app's router; with it, the app boots and renders its not-found page.
echo "==> Emitting 404.html (app-shell fallback for unknown URLs) ..."
"$repo_root/scripts/emit-404.sh" "$public_dir"

echo "==> Refreshing $repo_root/docs from $public_dir ..."
mkdir -p "$repo_root/docs"
find "$repo_root/docs" -mindepth 1 -maxdepth 1 -exec rm -rf {} +
cp -r "$public_dir"/. "$repo_root/docs/"
touch "$repo_root/docs/.nojekyll"

echo "==> Done. Review with 'git status'/'git diff --stat -- docs', then commit and push to main."
echo "==> Row 324: $target_dir stays warm for the next deploy. The no-base-path validation tree follows Cargo.lock/toolchain changes only: scripts/lane-target.sh warm-release (it says when it is current)."
