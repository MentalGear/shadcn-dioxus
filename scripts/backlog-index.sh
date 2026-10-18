#!/usr/bin/env bash
#
# backlog-index.sh -- rebuild dev-docs/backlog.md (the index) from the row files in dev-docs/backlog/.
#
# The row files are the source of truth (one per backlog row: dev-docs/backlog/<NNN>-<slug>.md, or
# dev-docs/backlog/done/ once done; narrative sections in dev-docs/backlog/notes/). After adding or editing a
# row, run this: it syncs each row's `status`/`size` front matter from its sections and rewrites the generated
# tables of dev-docs/backlog.md. `--check` writes nothing and exits 1 on any problem (that is what
# scripts/check-backlog-index.sh runs).
#
# Usage: scripts/backlog-index.sh [--check]
set -uo pipefail
cd "$(dirname "$0")/.."
exec python3 -I scripts/backlog-index.py "$@"
