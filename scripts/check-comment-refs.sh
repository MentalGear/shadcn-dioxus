#!/usr/bin/env bash
#
# check-comment-refs.sh  (backlog row 266 (a); rule and how-to: dev-docs/history-in-comments.md)
#
# History is linked, not retold, so every LINK in a code comment has to resolve. Two checks over the comments
# (the same extractor as check-history-comments.sh, scripts/comment-scan.py):
#   row N    every `row N` / `rows N, M` / `backlog row N` resolves to dev-docs/backlog/NNN-*.md or
#            dev-docs/backlog/done/NNN-*.md. A `row N` counts as a backlog row when its comment block mentions
#            "backlog", or N is a 3-digit id within 25 of the highest one (a grid's "row 998" is a data row).
#   path     every repo path cited under dev-docs/ scripts/ primitives/ preview/ playwright/ templates/ labs/ docs/
#            cli/ dioxus-attributes/ test-harness/ .github/ exists (a `:123` line suffix is ignored; a name
#            wrapped at a hyphen onto the next line, a placeholder (`<name>`, `*`, `{..}`, `...`) and
#            node_modules/ preview/public/ target/ are skipped).
# `docs/` is the DEPLOYED SITE; the project docs moved to `dev-docs/`, so `docs/backlog.md` in a comment is dead
# (the sweep fixes those with one sed; they are most of the baseline).
#
# Dead references that existed when the gate landed are in the shrink-only `scripts/check-comment-refs.baseline.tsv`
# (`file<TAB>kind<TAB>ref`; no line numbers, so unrelated edits do not disturb it). It is a DEBT REGISTER, not an
# approval: fix the citation. The gate fails on a dead reference not in the baseline, and on a baseline entry that
# now resolves (run `--update-baseline` to lock the shrink in). A deliberate one carries `ref-ok: <reason>`.
#
# Usage:
#   scripts/check-comment-refs.sh                  check against the baseline, then run the self-test
#   scripts/check-comment-refs.sh --inventory      print every unresolved reference with file:line
#   scripts/check-comment-refs.sh --update-baseline
#       rewrite the baseline. Refuses to ADD an entry (shrink-only) unless the baseline does not exist yet or
#       --allow-grow is also given (say why in the commit).
#   scripts/check-comment-refs.sh --self-test      mutation tests (dead row, dead path in rust/css/ts/sh ...)
# Exit status: 0 clean; 1 a new dead reference / stale baseline; 2 bad usage.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
case "${1:-}" in
    -h|--help) sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
esac
if [ "$#" -gt 0 ]; then exec python3 -I scripts/comment-scan.py refs "$@"; fi
# The default run (the gate) also proves the gate can turn red: if the self-test rots, a regression is green again.
python3 -I scripts/comment-scan.py refs || exit $?
out=$(python3 -I scripts/comment-scan.py refs --self-test 2>&1) || { printf '%s\n' "$out"; echo "check-comment-refs: FAIL -- --self-test failed"; exit 1; }
