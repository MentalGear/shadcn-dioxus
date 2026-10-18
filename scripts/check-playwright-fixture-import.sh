#!/usr/bin/env bash
#
# check-playwright-fixture-import.sh
#
# Every Playwright spec must take its runtime `test`/`expect` from
# `playwright/fixtures.ts`, never straight from "@playwright/test".
#
# fixtures.ts is the suite's one import point for the runtime `test`/`expect`/
# `devices`. It used to force `html { scroll-behavior: auto !important }` on
# every context (dev-docs/backlog.md row 110); row 228 deleted that override
# because it hid the ambient-smooth navigation bug from every spec, so today
# it is a plain re-export. The gate stays so a future suite-wide fixture
# reaches every spec: a fixture only reaches tests declared through the
# extended `test` object, so a spec that imports the stock one silently opts
# out -- which happened once already, to navbar.spec.ts, the day the first
# fixture landed. Type-only imports
# (`import type { Page } ...`, `import { type Page } ...`) are fine.

set -euo pipefail
cd "$(dirname "$0")/.."

bad=0
while IFS= read -r file; do
  # Collapse each import statement onto one line so multi-line braces match.
  while IFS= read -r stmt; do
    names=$(printf '%s' "$stmt" | sed -E 's/^import[[:space:]]*\{([^}]*)\}.*/\1/')
    for name in $(printf '%s' "$names" | tr ',' ' '); do
      case "$name" in
        test|expect|devices)
          echo "$file: imports runtime '$name' from \"@playwright/test\" -- import it from fixtures.ts instead"
          bad=1 ;;
      esac
    done
  done < <(tr '\n' ' ' < "$file" | grep -oE "import[[:space:]]*\{[^}]*\}[[:space:]]*from[[:space:]]*['\"]@playwright/test['\"]" || true)
done < <(find playwright -name '*.spec.ts' -not -path '*/node_modules/*')

if [ "$bad" -ne 0 ]; then
  exit 1
fi
echo "check-playwright-fixture-import: OK"
