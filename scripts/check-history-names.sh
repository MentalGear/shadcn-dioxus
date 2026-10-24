#!/usr/bin/env bash
#
# check-history-names.sh  (backlog row 384; the names half of dev-docs/history-in-comments.md)
#
# A name describes what a test asserts, in the present tense; the story of the bug behind it lives in the backlog row
# and is linked from a comment. This gate keeps that true of NAMES, which check-history-comments.sh cannot see: a
# Playwright `test(` / `test.describe(` / `describe(` / `it(` / `test.step(` title, a Rust `#[test]` function name and a
# spec file name must not carry a history marker -- an ISO date 2026-08 or later, "regression" / "(regression guard)",
# "round 5" / "batch 13" / `parity3`, "user-reported" / "owner's", "pre-fix" / "before the fix", "the old behaviour",
# "no longer", "follow-up", or a "(... lane)" tag. It reuses comment-scan.py's phrase tables (`history_reason`: "used to",
# "previously", "now uses", SHAs, ...) and adds the name-only forms (NAME_MARKERS in scripts/history-names.py).
#
# Before/after CONTRAST is history too (round 3): a title that says what a change did not move compares the code with its
# earlier shape. `contrast_reason` in scripts/history-names.py flags `still <verb>` ("selecting a sub-item still closes
# the whole menu", "hovering still scrolls"), `is unchanged:` / a title that leads with `unchanged` / `(unchanged)` /
# `is unchanged` with no operation in the title ("keyboard is unchanged: ArrowRight ..."), `regained`, `is restored`
# ending a clause and `works/passes/green again`. A guarantee under a condition ("the scrim blocks pointer events while
# disabled", "... still over it" after `while`, "under RTL ... still wraps", "after X the fade still follows"), `still`
# before an adjective or participle ("still popover-open", "a still shape"), `and still ...` continuing a predicate,
# `the order is unchanged` after `Escape cancels a lift`, and a Rust `..._is_unchanged` (an identity test) stay. The
# unchanged rules read titles only, never Rust or file names.
#
# A separate gate and not a mode of check-history-comments.sh, because the failure shape differs: comments are
# counted against a shrink-only per-file baseline (old text is debt), while a name is clean or it is not, so there is
# no baseline: any hit fails, and the only exemption is a reasoned ALLOW entry (path, name substring, reason) in
# scripts/history-names.py, which fails once stale. A cited repo path ("dev-docs/research/x-2026-09-19.md") is a link
# and is masked before the rules run; a `row N` pointer in a title is a link too and stays.
#
# Usage:
#   scripts/check-history-names.sh               check the tree, then run the self-test
#   scripts/check-history-names.sh --inventory   print every flagged name: file:line, kind, reason, name
#   scripts/check-history-names.sh --self-test   must-flag / must-not-flag tables, extraction cases, and mutations on
#       throwaway repos and on a real spec (a dated title, a "regression" describe, a batch-named spec file, a Rust
#       `#[test]` named for a regression, a stale or reasonless ALLOW entry) that must turn the gate red
# Exit status: 0 clean; 1 a name carries history or an ALLOW entry is stale; 2 bad usage.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
case "${1:-}" in
    -h|--help) sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
esac
if [ "$#" -gt 0 ]; then exec python3 -I scripts/history-names.py "$@"; fi
# The default run (the gate) also proves the gate can turn red: if the self-test rots, a regression is green again.
python3 -I scripts/history-names.py || exit $?
out=$(python3 -I scripts/history-names.py --self-test 2>&1) || { printf '%s\n' "$out"; echo "check-history-names: FAIL -- --self-test failed"; exit 1; }
