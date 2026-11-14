#!/usr/bin/env bash
#
# check-suite-reuse-e2e.sh  (backlog row 513)
#
# Drives the real playwright/tools/run-suite.sh --reuse with the real Playwright runner on a scratch git repo with planted specs
# and a planted site (no build, no browser) and checks that only what should run runs and that nothing wrong is accepted: a
# spec-only fix reuses the rest; a helper change re-runs only its importers (a spec that reads it by path is the pinned, accepted
# residue); a failing test, a test.only, a test that interrupts its runner, a runtime skip, a tool edit and an environment read
# through globalThis.process are each handled as the suite key's rules say; an edit made and reverted in the shared checkout
# during a run does not reach it (the run reads a private worktree) and the deploy's tested-tree check refuses that checkout.
#
# The result is reused for identical inputs through the gate kit's store (scripts/gate-reuse.sh): the key is the hash of
# playwright/tools/, the Playwright config files, the scripts the run uses (including scripts/ts-tools's manifest and lock, the
# parser of the keys, and scripts/gate-reuse.sh, the store) and the Node version, so the run (about 40 s) happens
# only when one of those changed. The script holds the shared Playwright lock (pw.lock) for the run and gives up after
# E2E_LOCK_WAIT seconds (default 900) with a failure that says the lock was busy, which is not a verdict on the code.
#
# Usage: scripts/check-suite-reuse-e2e.sh        (GATE_REUSE=0 forces the run)    Exit status: 0 = OK; 1 = a promise broke.
set -uo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
# shellcheck source=scripts/gate-reuse.sh
. "$repo_root/scripts/gate-reuse.sh"
command -v node >/dev/null || { echo "check-suite-reuse-e2e: node is required"; exit 1; }
[ -d playwright/node_modules ] || { echo "check-suite-reuse-e2e: playwright/node_modules is missing (pnpm install --frozen-lockfile in playwright/)"; exit 1; }

inputs() {
  git ls-files -co --exclude-standard playwright/tools scripts/lib/pinned-worktree.sh scripts/emit-404.sh scripts/serve-spa.py scripts/gate-reuse.sh scripts/check-suite-reuse-e2e.sh scripts/ts-tools/package.json scripts/ts-tools/pnpm-lock.yaml
  printf '%s\n' playwright/base-url.ts playwright/sandbox-chromium.cjs playwright/suite.local.config.ts playwright/package.json playwright/pnpm-lock.yaml playwright/pnpm-workspace.yaml playwright/.npmrc
}
key="$( { echo "format 1"; node --version; inputs | sort -u | while IFS= read -r f; do [ -f "$f" ] && printf '%s %s\n' "$f" "$(git hash-object "$f")"; done; } | sha1sum | cut -c1-40)"
if [ "${GATE_REUSE:-1}" != 0 ] && hit="$(gate_reuse_lookup "$key" suite-reuse-e2e)"; then
  echo "check-suite-reuse-e2e: OK (reused: identical inputs passed on tree $(printf '%s' "$hit" | cut -f1))"
  exit 0
fi

lock="${LANE_TGT:-/home/user/tgt}/pw.lock"
start=$(date +%s)
log="$(mktemp)"; trap 'rm -f "$log"' EXIT
flock -w "${E2E_LOCK_WAIT:-900}" "$lock" env SUITE_E2E_PRIVATE_LOCK=1 playwright/tools/suite-reuse-e2e.sh > "$log" 2>&1
rc=$?
if [ "$rc" = 1 ] && ! grep -q "suite-reuse-e2e:" "$log"; then
  echo "check-suite-reuse-e2e: FAIL -- the Playwright lock $lock was busy for ${E2E_LOCK_WAIT:-900} s; the scenario did not run (not a verdict on the code)"; exit 1
fi
tail -3 "$log"
if [ "$rc" != 0 ]; then sed -n "/FAIL/p" "$log"; gate_reuse_forget "$key" suite-reuse-e2e; echo "check-suite-reuse-e2e: FAIL"; exit 1; fi
gate_reuse_record "$key" suite-reuse-e2e "$(git rev-parse --short HEAD)" "$(( $(date +%s) - start ))" || true
echo "check-suite-reuse-e2e: OK"
