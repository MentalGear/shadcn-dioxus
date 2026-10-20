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
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
sel=(node tools/select.mjs --format list)
[ -n "${CHANGED_RANGE:-}" ] && sel+=(--range "$CHANGED_RANGE")
[ -n "${CHANGED_MATRIX:-}" ] && sel+=(--matrix)
mapfile -t specs < <("${sel[@]}")
if [ "${#specs[@]}" -eq 0 ]; then
  echo "test-changed: no spec covers the diff (docs, gates or tooling only) -- nothing to run"
  exit 0
fi
echo "test-changed: ${#specs[@]} spec file(s) cover the diff: ${specs[*]}" >&2
exec npx playwright test -c ssg.local.config.ts "${specs[@]}" "$@"
