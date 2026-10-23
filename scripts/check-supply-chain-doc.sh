#!/usr/bin/env bash
# Wrapper: the gate lives with the playbook so dev-docs/supply-chain/ can be extracted on its own
# (`git subtree split`); this file is this repository's wiring, the name scripts/run-gates.sh lists.
exec "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/dev-docs/supply-chain/check-supply-chain-doc.sh" "$@"
