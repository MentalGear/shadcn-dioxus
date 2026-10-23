#!/usr/bin/env bash
# Builds the preview/gallery app and publishes it into /docs for GitHub
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
#   DEPLOY_SUITE_WORKERS (default 3: the main loop's full-run setting), DEPLOY_SUITE_CHUNKS (default 6),
#   DEPLOY_SUITE_PORT (default 4830).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
suite_only=0
case "${1:-}" in
  "") ;;
  --suite-only) suite_only=1 ;;
  *) echo "deploy-preview: unknown argument $1 (usage: scripts/deploy-preview.sh [--suite-only])" >&2; exit 2 ;;
esac

lane_tgt="${LANE_TGT:-/home/user/tgt}"
suite_dir="$lane_tgt/release-suite"

# Prints the summary of a run-suite.sh output directory and exits non-zero unless every chunk wrote its JSON report
# and no test failed (`unexpected`). The list of failing titles goes to stderr.
suite_verdict() {
  node - "$1" "$2" <<'NODE'
const fs = require("fs");
const [dir, chunks] = process.argv.slice(2);
let expected = 0, unexpected = 0, flaky = 0, skipped = 0;
const missing = [], failed = [], flakyTitles = [];
const walk = (suite, file) => {
  for (const s of suite.suites ?? []) walk(s, s.file ?? file);
  for (const spec of suite.specs ?? []) for (const t of spec.tests ?? []) {
    const title = `${spec.file ?? file}: ${spec.title}`;
    if (t.status === "unexpected") failed.push(title);
    if (t.status === "flaky") flakyTitles.push(title);
  }
};
for (let i = 1; i <= Number(chunks); i++) {
  const f = `${dir}/chunk-${i}.json`;
  if (!fs.existsSync(f)) { missing.push(i); continue; }
  let j;
  try { j = JSON.parse(fs.readFileSync(f, "utf8")); } catch { missing.push(i); continue; }
  expected += j.stats.expected; unexpected += j.stats.unexpected; flaky += j.stats.flaky; skipped += j.stats.skipped;
  for (const s of j.suites ?? []) walk(s, s.file);
  if ((j.errors ?? []).length) missing.push(`${i} (${j.errors.length} run errors)`);
}
console.log(`passed=${expected} failed=${unexpected} flaky=${flaky} skipped=${skipped} chunks_missing=${missing.length ? missing.join(",") : 0}`);
if (flakyTitles.length) console.error(`flaky (passed on retry):\n  ${flakyTitles.join("\n  ")}`);
if (failed.length) console.error(`failed:\n  ${failed.join("\n  ")}`);
process.exit(unexpected === 0 && missing.length === 0 && expected > 0 ? 0 : 1);
NODE
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
  local tgt pub server_pid=""
  echo "==> Release suite (row 322): no green run recorded for tree $tree ($head); building and running it."
  mkdir -p "$suite_dir"; rm -rf "$out"; mkdir -p "$out"
  tgt="$(cd "$repo_root" && scripts/lane-target.sh create "$name" --release | sed -n 's/^export CARGO_TARGET_DIR=//p')"
  [ -n "$tgt" ] || { echo "deploy-preview: could not copy the warm release-template tree (scripts/lane-target.sh create $name --release)" >&2; exit 1; }
  # shellcheck disable=SC2064  # expand now: the trap must remove THIS copy and stop THIS server
  trap "[ -n \"\${server_pid:-}\" ] && kill \"\${server_pid}\" 2>/dev/null; rm -rf '$tgt'" EXIT
  ( cd "$repo_root" && CARGO_TARGET_DIR="$tgt" CARGO_INCREMENTAL=0 scripts/lane-lock.sh gate --label "deploy-suite:build" -- \
      scripts/build-ssg.sh release ) > "$out/build.log" 2>&1 \
    || { tail -30 "$out/build.log" >&2; echo "deploy-preview: the release-suite build failed (log $out/build.log)" >&2; exit 1; }
  pub="$tgt/dx/preview/release/web/public"
  "$repo_root/scripts/emit-404.sh" "$pub" >/dev/null
  python3 "$repo_root/scripts/serve-spa.py" "$pub" "$port" >/dev/null 2>&1 9>&- 3>&- </dev/null &
  server_pid=$!
  for _ in $(seq 1 50); do curl -fsS -o /dev/null "http://127.0.0.1:$port/" && break; sleep 0.2; done
  ( cd "$repo_root/playwright" && PLAYWRIGHT_BASE_URL="http://127.0.0.1:$port" PW_BUILD_PROFILE=release SSG_SITE_DIR="$pub" \
      tools/run-suite.sh --out "$out" --chunks "$chunks" --workers "$workers" ) | tee "$out/run-suite.log" || true  # the verdict below decides
  kill "$server_pid" 2>/dev/null || true; server_pid=""
  rm -rf "$tgt"; trap - EXIT
  local verdict
  if verdict="$(suite_verdict "$out" "$chunks")"; then
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
# (~3-5 min) instead of the 700-crate dependency tree (~22-50 min). Without a /home/user/tgt it falls back to the
# workspace's own `target/` as before. The no-base-path validation tree (`release-template`, see
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

# `dx build --ssg` has a known, pre-existing race (unrelated to any one
# source change -- reproduced identically on an untouched checkout, see
# session notes) where its SSG render pass fires all routes' requests at
# the freshly-started fullstack server concurrently, and some of those
# requests can land before something the server needs to inject the
# hydration bootstrap `<script type="module">` is ready. The affected
# route's static HTML then has no script tag at all -- not a visible
# error, just a dead, never-hydrates page if it ships.
#
# Earlier note in this file claimed a cold cargo cache "reliably self-heals"
# this -- that was wrong, disproved by direct testing (2026-09-18 docs
# deploy session): two independent fully-cold rebuilds (all of
# target/wasm-release, target/wasm32-unknown-unknown/wasm-release,
# target/server-release, target/x86_64-*/server-release, and
# target/dx/preview/release cleared first -- confirmed by mtime, not just
# assumed) reproduced the *identical* 5-of-6 failure, at near-identical
# internal timing both times. Compile-cache warmth was never the variable.
# The actual cause: by default `dx build --fullstack` compiles the server
# and client targets in parallel, and the client side's post-compile work
# (wasm-bindgen, wasm-opt -- both CPU-heavy) is still competing for CPU
# when the SSG pass's concurrent requests hit the freshly-listening server,
# starving whatever it needs to finish before it can inject the script tag.
# On this sandbox's 4 cores, `--force-sequential=true` above (server build,
# then client build, so nothing competes with the server during its SSG
# window) fixed it 2/2 clean runs after 2/2 failures without it -- dx's own
# `--help` already default-enables this under `CI=1` for what's presumably
# this same reason. The verification below stays as a safety net regardless
# (belt-and-suspenders: don't ship a dead page even if this analysis is
# ever wrong for some other environment), on every route dx actually
# produced, rather than trust that the flag always suffices.
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
  echo "did NOT fix this when it was tested (see the comment above) -- diagnose" >&2
  echo "fresh from this build's own log rather than assume that remedy." >&2
  exit 1
fi
echo "==> OK: all $route_count route(s) have their bootstrap script."

# dev-docs/backlog.md row 46: before the path-segment route construction
# (preview/src/main.rs's ComponentDemoPath/ComponentBlockDemoPath), every
# `/component/?name=X&` page prerendered as the "Component not found" shell
# for every X, because a query-string route is not enumerable at build time
# -- this check is the safety net for a recurrence of that class, on every
# route dx actually produced, the same belt-and-suspenders shape as the
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
