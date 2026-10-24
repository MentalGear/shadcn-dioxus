#!/usr/bin/env bash
#
# check-gates-doc.sh
#
# dev-docs/gates.md, the one page that says what every gate checks, is generated (scripts/gates-doc.py) from the gate
# list in scripts/run-gates.sh and the header of each scripts/check-*.sh, so it can never drift and a new gate never
# needs a prose edit in CLAUDE.md or lane-rules.md. This gate fails when the committed page differs from the generated
# text (run `scripts/gates-doc.py --write` and commit it), when a gate sits above no `# group:` line, when a check
# script's first header paragraph is too thin (under 100 characters), too long (over 1100) or ends in a colon, and
# when a gate that is not a check script has no comment lines above its entry in run-gates.sh. Its self-test proves
# each of those is reported.
#
# Usage: scripts/check-gates-doc.sh
# Exit status: 0 when clean, 1 on a violation.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
python3 -I scripts/gates-doc.py --self-test >/dev/null || { python3 -I scripts/gates-doc.py --self-test >&2; echo "check-gates-doc: self-test FAILED" >&2; exit 1; }
python3 -I scripts/gates-doc.py --check
