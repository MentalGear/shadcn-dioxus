#!/usr/bin/env bash
# Run only the Playwright specs that cover what this branch changed (owner 2026-10-20, row 152a follow-up).
#
#   npm run test:changed                         # from playwright/: diff = origin/main...HEAD + uncommitted changes
#   npm run test:changed -- --workers=2          # extra args go to `playwright test`
#   CHANGED_RANGE=abc123..HEAD npm run test:changed
#   CHANGED_MATRIX=1 npm run test:changed        # also the route-matrix specs for a single-component change
#
# The spec list comes from tools/select.mjs (the cover relation documented at its top: a component's own specs, a
# primitive's consumer components' specs, the broad set for shared plumbing, every spec for a config change). Uses
# ssg.local.config.ts, so PLAYWRIGHT_BASE_URL must point at a served build of the tree under test (scripts/lane-ssg.sh).
# This is the lane's and the small-PR's run; the full suite still runs once per merge batch (dev-docs/lane-rules.md).
#
# A changed theme or site stylesheet (preview/assets/*.css) is read rule by rule (backlog row 323): select.mjs crawls
# PLAYWRIGHT_BASE_URL (or reads SELECT_SSG_DIR, the lane's pub dir) to learn which pages carry the classes the changed
# rules touch, and the matrix specs whose tests are one-per-page run only the tests of the pages reached -- the rest are
# dropped with the --grep-invert pattern select.mjs prints in its json. Anything it cannot place keeps the whole BROAD
# set. CHANGED_FULL_CSS=1 forces that (select.mjs --no-css-narrow); `node tools/select.mjs --explain` says why.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
sel=(node tools/select.mjs --format json)
[ -n "${CHANGED_RANGE:-}" ] && sel+=(--range "$CHANGED_RANGE")
[ -n "${CHANGED_MATRIX:-}" ] && sel+=(--matrix)
[ -n "${CHANGED_FULL_CSS:-}" ] && sel+=(--no-css-narrow)
json="$("${sel[@]}")"
field() { printf '%s' "$json" | node -e 'const j=JSON.parse(require("fs").readFileSync(0,"utf8"));const v=j[process.argv[1]];if(process.argv[1]==="specs")process.stdout.write(v.join("\n")+(v.length?"\n":""));else if(process.argv[1]==="grepInvert")console.log(v||"");else console.log((v||[]).map((n)=>`${n.file} ${n.kept}/${n.total}`).join(", "))' "$1"; }
mapfile -t specs < <(field specs)
grep_invert="$(field grepInvert)"
narrowed="$(field narrowed)"
if [ "${#specs[@]}" -eq 0 ]; then
  echo "test-changed: no spec covers the diff (docs, gates or tooling only) -- nothing to run"
  exit 0
fi
echo "test-changed: ${#specs[@]} spec file(s) cover the diff: ${specs[*]}" >&2
[ -n "$narrowed" ] && echo "test-changed: tests kept of the page-scoped matrix specs (CSS reaches only some pages): $narrowed" >&2
extra=()
[ -n "$grep_invert" ] && extra+=(--grep-invert "$grep_invert")
exec npx playwright test -c ssg.local.config.ts "${specs[@]}" ${extra[@]+"${extra[@]}"} "$@"
