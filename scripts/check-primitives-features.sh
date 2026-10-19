#!/usr/bin/env bash
#
# check-primitives-features.sh
#
# `dioxus-primitives` has optional features so that an app compiles only what its components use
# (dev-docs/backlog.md row 217): `router` (Navbar; dioxus-router, ~40 crates), `calendar` (Calendar + DatePicker;
# `time`), `color-picker` (ColorPicker; `palette` and 11 companions). Inside this workspace every
# build unifies ALL of them (preview turns them on), so a gated module that quietly depends on another gated
# crate -- or a non-gated module that grew a `use time::` -- compiles here and breaks in a user's tree that enabled
# only the feature its component.json asks for. This gate builds the crate ON ITS OWN, outside that unification:
#   - no default features and no optional feature at all (the Button-only app), including tests and benches;
#   - each optional feature alone (+ `web`, the browser arm, which must not drag `time` in on its own);
#   - every optional feature together with `web`.
# Which features a COMPONENT must ask for is `scripts/check-cargo-dep-versions.sh`; the whole install is
# `scripts/registry-install-smoke.sh`.
#
# Needs an absolute CARGO_TARGET_DIR (run-gates.sh sets it) and compiles dioxus-primitives, so it is a cargo gate.
#
# Usage: CARGO_TARGET_DIR=/abs/dir scripts/check-primitives-features.sh
# Exit status: 0 when every subset builds.

set -uo pipefail
cd "$(dirname "$0")/.."

case "${CARGO_TARGET_DIR:-}" in
  /*) ;;
  *) echo "check-primitives-features: CARGO_TARGET_DIR must be set to an absolute, isolated dir" >&2; exit 2 ;;
esac

# Every optional feature is read from the manifest, so a new one is covered the day it appears: all features
# except `default`, `web` and the umbrella ones that only forward to another primitives feature.
optional="$(python3 -I - <<'PY'
import tomllib
f = tomllib.load(open("primitives/Cargo.toml", "rb"))["features"]
print(" ".join(k for k in f if k not in ("default", "web")))
PY
)"
[ -n "$optional" ] || { echo "check-primitives-features: no optional features found in primitives/Cargo.toml" >&2; exit 2; }

fail=0
run() {
  local label="$1"; shift
  if out="$(cargo check -p dioxus-primitives --no-default-features "$@" 2>&1)"; then
    echo "check-primitives-features: ok   $label"
  else
    echo "check-primitives-features: FAIL $label" >&2
    echo "$out" | tail -n 25 | sed 's/^/   | /' >&2
    fail=1
  fi
}

run "(no features) --all-targets" --all-targets
for f in $optional; do run "$f" --lib --features "$f"; done
run "web" --lib --features web
run "web + $(echo $optional | tr ' ' '+')" --lib --features "web,$(echo $optional | tr ' ' ',')"

[ "$fail" -ne 0 ] && exit 1
echo "check-primitives-features: OK -- dioxus-primitives builds with no features, with each of [$optional] alone, with web, and with all of them."
