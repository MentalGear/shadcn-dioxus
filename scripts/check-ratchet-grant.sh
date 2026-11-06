#!/usr/bin/env bash
#
# check-ratchet-grant.sh  (backlog row 444, phase 0.1; the rule: dev-docs/js-interop.md, "The ratchet")
#
# Proves the grant rule of `scripts/lib/ratchet.py`, which every shrink-only baseline gate (check-js-volume, check-native-slots, the TypeScript scan's callers) reads: a raise passes only against an open row at the base with a large enough `js-grant`, and a grant in the lane's own diff, a closed row, a smaller grant, a wrong total and no base under `RATCHET_REQUIRE_BASE` are each red.
#
# Runs `python3 -I scripts/lib/ratchet.py --self-test` (scenarios on throwaway git repos) and, from this same wrapper, `python3 -I scripts/lib/platform_rules.py --self-test`, the fixtures of the platform-rule TSV reader (`primitives/conformance/platform-rules.tsv`), which has no gate of its own yet.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
case "${1:-}" in
    -h|--help) sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
esac

python3 -I scripts/lib/ratchet.py --self-test
python3 -I scripts/lib/platform_rules.py --self-test
