#!/usr/bin/env bash
#
# backlog-close.sh -- close backlog rows: refuse while something still names a row, else move it to
# dev-docs/backlog/done/, repair the links, set its status and rebuild the indexes (scripts/backlog-close.py has the rules).
#
# Usage: scripts/backlog-close.sh <id>... [--status "LANDED in <ref>"] [--grant-spent]
set -uo pipefail
cd "$(dirname "$0")/.."
exec python3 -I scripts/backlog-close.py "$@"
