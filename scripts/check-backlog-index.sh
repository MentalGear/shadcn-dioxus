#!/usr/bin/env bash
#
# check-backlog-index.sh
#
# The backlog is one file per row (dev-docs/backlog/<NNN>-<slug>.md, finished rows in dev-docs/backlog/done/,
# narrative sections in dev-docs/backlog/notes/), and dev-docs/backlog.md is only the generated INDEX over them
# (about 250 code and doc comments cite "dev-docs/backlog.md row N"; the index resolves each to its file). A
# hand edit of the index, or a row edited without regenerating, would make the index lie, so by construction:
#   - every row file has the front matter fields id, title, status, size, area, updated (and the four
#     sections Item / Status / trigger / Size / Detail, in order);
#   - ids are unique and match the zero-padded file name prefix; no row exists in both open and done;
#   - `status` and `size` equal what is derived from the sections (scripts/backlog-index.sh syncs them);
#   - the committed dev-docs/backlog.md equals the one generated from the row files;
#   - every relative link in dev-docs/**/*.md and the root CLAUDE.md/README.md resolves to a file (pure
#     filesystem checks, no network).
#
# To change a row: edit its file, run scripts/backlog-index.sh, commit both.
#
# Usage: scripts/check-backlog-index.sh
# Exit status: 0 when clean, 1 on a violation.
set -uo pipefail
cd "$(dirname "$0")/.."
rc=0
python3 -I scripts/backlog-index.py --check || rc=1
python3 -I scripts/check-doc-links.py || rc=1
exit $rc
