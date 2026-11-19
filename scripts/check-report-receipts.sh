#!/usr/bin/env bash
#
# check-report-receipts.sh
#
# scripts/report-receipts.py is the checker the main loop runs over a delegated report's ledger before it relays a finding
# (agent-rules/delegation-and-models.md): it confirms each quote, re-runs each count and sorts every claim into
# VERIFIED, LOGGED-ONLY, OPINION, UNBACKED, BROKEN or UNVERIFIABLE. This gate runs its self-test, a fixture repo and
# ledgers with one claim per status and per receipt failure, so the checker cannot rot unnoticed between two uses.
#
# Usage: scripts/check-report-receipts.sh
# Exit status: 0 when every fixture holds, 1 otherwise.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
python3 -I scripts/report-receipts.py --self-test || { echo "check-report-receipts: self-test FAILED" >&2; exit 1; }
