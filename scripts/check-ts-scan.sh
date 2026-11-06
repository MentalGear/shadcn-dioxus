#!/usr/bin/env bash
#
# check-ts-scan.sh  (backlog row 444, phase 0.1; the scanner and its rules: scripts/ts-scan/scan.mjs)
#
# TypeScript compiler-API scan of `primitives/src/ts/**`: computed access with a non-literal key outside `own()`, string sinks (`eval`, `Function`, `setTimeout`/`setInterval` with a string), dynamic-key `Reflect`/`Object` calls, `.constructor`, the verb-row cap, `any`/`as any` against a shrink-only baseline (`scripts/ts-scan/bridge.baseline.tsv`), and the hard rules under `ts/behaviours/` (free-identifier allowlist, banned members, import allowlist, no `@ts-ignore`).
#
# Runs `node scripts/ts-scan/scan.mjs` on the tree and then `--self-test`, which plants each shape in a fixture (`scripts/ts-scan/fixtures/`) and requires it red. The compiler is resolved in one place, `TS_MODULE` in scan.mjs (the sandbox image's /opt/node-tools, `TS_SCAN_TYPESCRIPT` overrides it); a missing compiler is red, never a silent pass.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
case "${1:-}" in
    -h|--help) sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
esac

command -v node >/dev/null || { echo "check-ts-scan: node is not installed" >&2; exit 1; }
node scripts/ts-scan/scan.mjs
node scripts/ts-scan/scan.mjs --self-test
