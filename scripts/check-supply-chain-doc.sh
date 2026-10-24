#!/usr/bin/env bash
# The controls table of the supply-chain playbook (`dev-docs/supply-chain/`) is true of the tree: a row marked
# `enforced` or `manual` names only script and config paths that exist, a `planned` row's script does not exist yet,
# every script in the gate's declared set has a row, a `run-gates` row is in this gate list, ids and counts agree, and
# `SECURITY.md` links the playbook. The gate lives with the playbook so the directory can be extracted on its own
# (`git subtree split`); this file is this repository's wiring. It runs its own mutation self-test; no network.
#
# The scripts the playbook must cover are the gate's declared `SUPPLY_CHAIN_SCRIPTS` set (in
# `dev-docs/supply-chain/`).
exec "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/dev-docs/supply-chain/check-supply-chain-doc.sh" "$@"
