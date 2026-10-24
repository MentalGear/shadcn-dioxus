#!/usr/bin/env bash
# The SSG postcondition (`scripts/ssg-verify.sh`, which `scripts/build-ssg.sh` applies after every dx build) still
# rejects every broken-site shape it was written for -- a missing, stale, tiny or truncated page, a dx `ssg error:`
# line, a missing `SSG complete`, a truncated route list: the gate runs `ssg-verify.sh --self-test`. dx 0.7.9 exits 0
# for a site with failed routes, so the postcondition is the only thing that turns one red, and if its self-test rots
# a broken build is green again.
set -euo pipefail
cd "$(dirname "$0")/.."
out=$(bash scripts/ssg-verify.sh --self-test 2>&1) || { printf '%s\n' "$out"; echo "check-ssg-verify: FAIL -- ssg-verify.sh --self-test failed"; exit 1; }
echo "check-ssg-verify: OK ($(printf '%s\n' "$out" | tail -1))"
