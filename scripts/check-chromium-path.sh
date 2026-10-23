#!/usr/bin/env bash
#
# check-chromium-path.sh  (backlog row 374)
#
# Every local Playwright config, two specs and three scripts/*.mjs probes hard-coded the sandbox image's Chromium as
# revision 1194 under /opt/pw-browsers. A fresh container shipped revision 1243 (in `chrome-linux64/`, not
# `chrome-linux/`), so the configs without an exists-check failed to launch and the ones with it fell through to
# Playwright's own resolution, which wants the revision pinned by @playwright/test and found none either. The cure
# is one resolver, `playwright/sandbox-chromium.cjs` (`PW_CHROMIUM`, else the newest `chromium-<rev>` under
# `PLAYWRIGHT_BROWSERS_PATH`, else Playwright's default). This gate keeps it the only one:
#
# Rule 1. No file under playwright/ or scripts/ (not node_modules, test-results, playwright-report) names a
#         revisioned browser directory, `chromium-<digits>/` or `chromium_headless_shell-<digits>/`.
# Rule 2. Every playwright/*.config.ts that sets `executablePath` or `launchOptions` gets it from
#         sandbox-chromium.cjs (imports it), and no file in those trees sets `executablePath:` to a string literal.
#
# Usage: scripts/check-chromium-path.sh
# Exit status: 0 when clean, 1 on a violation.
set -euo pipefail
cd "$(dirname "$0")/.."

bad=0
prune=(-name node_modules -o -name test-results -o -name playwright-report)

while IFS= read -r hit; do
  echo "$hit -- hard-coded Chromium revision path; use playwright/sandbox-chromium.cjs (row 374)"
  bad=1
done < <(find playwright scripts \( "${prune[@]}" \) -prune -o -type f -print0 \
  | xargs -0 grep -nIE 'chromium(_headless_shell)?-[0-9]+/' 2>/dev/null || true)

while IFS= read -r hit; do
  echo "$hit -- executablePath set to a literal path; use playwright/sandbox-chromium.cjs (row 374)"
  bad=1
done < <(find playwright scripts \( "${prune[@]}" \) -prune -o -type f \( -name '*.ts' -o -name '*.js' -o -name '*.mjs' -o -name '*.cjs' \) -print0 \
  | xargs -0 grep -nE "executablePath:[[:space:]]*[\"'\`]" 2>/dev/null || true)

for cfg in playwright/*.config.ts; do
  if grep -qE 'executablePath|launchOptions' "$cfg" && ! grep -q 'sandbox-chromium.cjs' "$cfg"; then
    echo "$cfg: sets launchOptions/executablePath without importing ./sandbox-chromium.cjs (row 374)"
    bad=1
  fi
done

if [ "$bad" -ne 0 ]; then
  exit 1
fi
echo "check-chromium-path: OK"
