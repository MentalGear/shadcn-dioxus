#!/usr/bin/env bash
#
# check-dependency-register.sh
#
# Every direct dependency of the repo is evaluated once, in dev-docs/dependency-register.tsv (name, ecosystem, resolved version,
# outcome keep/lift/drop/exception, reach class, evidence date, D6 exception end date), and this gate keeps the register true of
# the tree. It fails on a declared dependency with no row, a row nothing declares, a version that differs from the lock, a
# malformed row, an exception whose end date is before today, and a build script or proc-macro that enters a user's install
# closure without being listed in dev-docs/dependency-register.closure.tsv.
#
# Declared means the registry dependencies of every Cargo.toml (workspace and detached crates; path and git dependencies are
# ours), every package.json (checked against its pnpm-lock.yaml too) and the cargo-installed dx. DEP_REGISTER_TODAY overrides
# the date for tests. The closure baseline is a shrink-only ratchet over `cargo tree -e normal,build` of dioxus-primitives (all
# features, wasm32 and host) and of each registry crate a component.json names; kinds come from `cargo metadata` and the tree's
# proc-macro mark. A listed crate that left the closure is red too (shrink it with scripts/dependency-register.py
# --closure-update). It sees what cargo resolves for the lock, not features a user's own app adds. The self-test mutates fixtures
# every way and needs no cargo; the tree checks use only `cargo tree` and `cargo metadata` with --offline --locked.
#
# Usage: scripts/check-dependency-register.sh
# Exit status: 0 when true, 1 on a violation, 2 when a source or a cargo call fails.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
exec python3 -I scripts/dependency-register.py "$@"
