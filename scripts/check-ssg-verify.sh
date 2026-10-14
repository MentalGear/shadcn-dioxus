#!/usr/bin/env bash
# Gate: the SSG postcondition (`scripts/ssg-verify.sh`, run by
# `scripts/build-ssg.sh` after every dx build) still rejects every broken-site
# shape it was written for -- missing, stale, tiny or truncated pages, a dx
# `ssg error:` line, a missing `SSG complete`, a truncated route list. dx 0.7.9
# exits 0 for a site with failed routes, so this check is the only thing that
# turns one red; if its self-test rots, a broken build is green again.
set -euo pipefail
cd "$(dirname "$0")/.."
out=$(bash scripts/ssg-verify.sh --self-test 2>&1) || { printf '%s\n' "$out"; echo "check-ssg-verify: FAIL -- ssg-verify.sh --self-test failed"; exit 1; }
echo "check-ssg-verify: OK ($(printf '%s\n' "$out" | tail -1))"
