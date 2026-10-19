#!/usr/bin/env bash
#
# check-docs-no-demo-list.sh
#
# A component page already renders every demo right below its docs, each with its own "About this example"
# notes (`variants/<variant>/notes.md`). A `## Demos` section in docs.md only repeated that list one line per
# demo (and drifted from it), so by construction no preview/src/components/*/docs.md may have an h2 whose text
# starts with "Demos". Put the prose about a demo in that variant's notes.md instead.
#
# Usage: scripts/check-docs-no-demo-list.sh
# Exit status: 0 when clean, 1 on a violation.
set -uo pipefail
cd "$(dirname "$0")/.."
hits=$(grep -nE '^##[[:space:]]+Demos' preview/src/components/*/docs.md)
if [ -n "$hits" ]; then
  echo "FAIL: docs.md has a 'Demos' h2 (move each demo's prose into variants/<variant>/notes.md):" >&2
  echo "$hits" >&2
  exit 1
fi
echo "OK: no docs.md has a 'Demos' h2"
