#!/usr/bin/env bash
#
# gen-component-manifests.sh
#
# Writes (or, with --check, verifies byte-for-byte) every component manifest from the component's own sources:
# `preview/src/components/<dir>/component.json` (exclude, globalAssets, componentDependencies, cargoDependencies),
# each `mod.rs`, and the root `component.json` members. `name`, `description` and `authors` stay hand-authored. The
# rules and the one input beyond the sources (the release `rev`) are in scripts/gen-component-manifests.py's header.
# Backlog row 430.
#
# Usage: scripts/gen-component-manifests.sh [--check]
# Exit status: 0 when written / identical, 1 when --check finds a stale file, 2 on a malformed input.

set -uo pipefail
cd "$(dirname "$0")/.."
exec python3 -I scripts/gen-component-manifests.py "$@"
