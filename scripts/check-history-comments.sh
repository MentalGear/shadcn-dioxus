#!/usr/bin/env bash
#
# check-history-comments.sh  (backlog row 266 (a); rule and how-to: dev-docs/history-in-comments.md)
#
# The codebase describes only its CURRENT state; history lives in the backlog row and is linked, not retold in
# a comment. A comment that says "used to", "previously", "no longer uses", "before the fix", "(2026-10-19)" or
# names a commit is a changelog entry in the wrong file: it goes stale the day the next fix lands, and it is
# the bulk of what a reader has to skip. This gate is a per-file RATCHET (the shape of
# check-cross-component-overrides.sh / check-props-docs.sh): it counts, per file, the comment lines that carry
# history, compares with `scripts/check-history-comments.baseline.tsv` (`path<TAB>count`), and fails when
#   - a file's count RISES above its baseline, or a new file appears with a non-zero count; or
#   - a count FELL (or a file went clean or was deleted) and the baseline still allows the old number: run
#     `--update-baseline` to lock the drop in (shrink-only, so history cannot creep back into a file that was cleaned).
#
# WHAT COUNTS (a comment line, per scripts/comment-scan.py: Rust `//` `///` `//!` `/* */`, CSS `/* */`, TS/JS `//`
# `/* */`, shell `#`; a marker inside a string is not a comment):
#   date     a 2026-08..2026-12 (or later) ISO date; older or 2026-0[1-7] dates are test data
#   sha      a 7-40 char hex token with a digit and two letters, not part of a path, `#colour`, `0x..` or `@sha`;
#            the pinned upstream revisions (aria-practices 7e4034b, shadcn 295a1f1) are provenance and stay
#   phrases  "used to" (not "is/can be used to"), "previously" (not "previously-focused"), "before the fix",
#            "formerly", "old behaviour/implementation/approach/name", "was changed/renamed/rewritten",
#            "we now", "now uses/reads/lives/owns/...", "no longer uses/needs/renders/...", "since removed"
# WHAT IS SKIPPED: dev-docs/ (history belongs there), docs/ (the deployed site), the vendored APG reference
# tree, generated primitives/src/js/*.js, *.md, lock files, and the two gates' own sources.
#
# An allow marker `history-ok: <reason>` on the comment line exempts it (a quoted upstream changelog, a
# provenance date the code needs). Prefer rewriting the line: say what the code IS and why, and link the row.
#
# Usage:
#   scripts/check-history-comments.sh                  check against the baseline, then run the self-test
#   scripts/check-history-comments.sh --inventory      print every counted line, file:line, reason, text
#   scripts/check-history-comments.sh --update-baseline
#       rewrite the baseline from the tree. Refuses to RAISE a count (shrink-only) unless the baseline does not
#       exist yet or --allow-grow is also given (say why in the commit).
#   scripts/check-history-comments.sh --self-test      mutation tests (dated comment, new file, stale, history-ok ...)
# Exit status: 0 clean; 1 a count rose / is stale; 2 bad usage.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
case "${1:-}" in
    -h|--help) sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
esac
if [ "$#" -gt 0 ]; then exec python3 -I scripts/comment-scan.py history "$@"; fi
# The default run (the gate) also proves the gate can turn red: if the self-test rots, a regression is green again.
python3 -I scripts/comment-scan.py history || exit $?
out=$(python3 -I scripts/comment-scan.py history --self-test 2>&1) || { printf '%s\n' "$out"; echo "check-history-comments: FAIL -- --self-test failed"; exit 1; }
