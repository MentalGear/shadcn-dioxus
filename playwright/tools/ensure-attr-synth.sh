#!/usr/bin/env bash
# Build playwright/oracle/attr-synth (hydration-parity.spec.ts' Rule 4c generator) THROUGH THE BUILD LOCK, once,
# before Playwright starts (backlog row 338). The spec must not run `cargo build --release` at module load: an unqueued
# ~3 min build, triggered by `playwright test --list` or any run that merely contains the file, would compete with gate
# and lane builds. So the spec only RUNS the binary and fails with a clear message when it is missing; this script is
# the one place that builds it (test-changed.sh when the spec is selected, run-suite.sh always).
#
#   playwright/tools/ensure-attr-synth.sh            # build if missing or older than its inputs
#   playwright/tools/ensure-attr-synth.sh --check    # exit 1 (no build) when it would build
#
# Inputs for freshness: attr-synth's own sources and manifest, and primitives/src (path dependency).
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
crate="$repo/playwright/oracle/attr-synth"
bin="$crate/target/release/attr-synth"
fresh() {
  [ -x "$bin" ] || return 1
  [ -z "$(find "$crate/src" "$crate/Cargo.toml" "$crate/Cargo.lock" "$repo/primitives/src" "$repo/primitives/Cargo.toml" -type f -newer "$bin" -print -quit 2>/dev/null)" ]
}
if fresh; then exit 0; fi
if [ "${1:-}" = "--check" ]; then echo "ensure-attr-synth: $bin missing or stale" >&2; exit 1; fi
echo "ensure-attr-synth: building attr-synth through the build lock" >&2
cd "$crate"
# Absolute, crate-local target dir (never an inherited CARGO_TARGET_DIR of the workspace build); jobs/incremental as lane-cargo.
exec "$repo/scripts/lane-lock.sh" lane --label "attr-synth build" -- \
  env CARGO_TARGET_DIR="$crate/target" CARGO_BUILD_JOBS="${LANE_CARGO_JOBS:-2}" CARGO_INCREMENTAL=0 \
  cargo build --release --quiet
