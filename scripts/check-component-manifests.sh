#!/usr/bin/env bash
#
# check-component-manifests.sh
#
# Every component's `component.json` (exclude, globalAssets, componentDependencies, cargoDependencies), every `mod.rs` and the
# root `component.json` members are the OUTPUT of `scripts/gen-component-manifests.sh`, computed from the component's installed Rust and CSS
# files, so a manifest cannot disagree with the files it ships (a missing `mod.rs` was E0583 and a missing crate E0433 only in a user's tree, and
# `registry-install-smoke.sh` existed to find them). The gate is the generator's `--check`: every generated file must be byte-identical to the tree;
# only `name`, `description` and `authors` are hand-authored. It then runs a mutation self-test on a scratch copy of the inputs (a hand-edited
# dependency, a deleted `mod.rs`, an edited `exclude`, an undeclared external crate, sibling component or effects class, a dropped root member)
# and requires each to turn the check red, so the check cannot rot into always-green. Backlog row 430. No network, no cargo.
#
# To change a manifest: edit the source it derives from (or `name`/`description`/`authors`), run scripts/gen-component-manifests.sh, commit both.
#
# Usage: scripts/check-component-manifests.sh
# Exit status: 0 when clean, 1 on a stale file or a self-test mutation that stayed green.

set -uo pipefail
cd "$(dirname "$0")/.."

scripts/gen-component-manifests.sh --check || exit 1

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
clean="$tmp/clean"
mkdir -p "$clean/preview/src" "$clean/primitives" "$clean/scripts"
cp -r preview/src/components "$clean/preview/src/"
cp preview/src/main.rs "$clean/preview/src/"
cp preview/Cargo.toml "$clean/preview/"
cp -r primitives/src primitives/Cargo.toml "$clean/primitives/"
cp Cargo.toml Cargo.lock component.json "$clean/"
cp scripts/gen-component-manifests.py "$clean/scripts/"

fail=0
# mutate <label> <command run inside the copy>: the generator's --check must go red.
mutate() {
  local label="$1"; shift
  rm -rf "$tmp/m"; cp -r "$clean" "$tmp/m"
  ( cd "$tmp/m" && eval "$@" ) >/dev/null 2>&1
  if ( cd "$tmp/m" && python3 -I scripts/gen-component-manifests.py --check ) >/dev/null 2>&1; then
    echo "check-component-manifests: self-test: \"$label\" left the check green" >&2
    fail=1
  fi
}
c=preview/src/components
# the clean copy itself must be green, else every red below proves nothing
if ! ( cd "$clean" && python3 -I scripts/gen-component-manifests.py --check ) >/dev/null 2>&1; then
  echo "check-component-manifests: self-test: the scratch copy of the inputs is not green" >&2; exit 1
fi
mutate "hand-added componentDependencies entry" "sed -i 's|\"globalAssets\"|\"componentDependencies\": [{ \"name\": \"card\", \"git\": \"x\" }], \"globalAssets\"|' $c/button/component.json"
mutate "deleted mod.rs" "rm $c/button/mod.rs"
mutate "mod.rs with a different shape" "printf 'pub mod component;\n' > $c/button/mod.rs"
mutate "edited exclude" "sed -i 's|\"variants\", ||' $c/button/component.json"
mutate "dropped theme asset" "sed -i 's|\"../../../assets/dx-components-theme.css\"||' $c/button/component.json"
mutate "undeclared external crate" "printf 'use time::Date;\n' >> $c/button/component.rs"
mutate "undeclared sibling component" "printf 'use crate::components::card::Card;\n' >> $c/button/component.rs"
mutate "undeclared relative sibling component" "printf 'use super::super::card::Card;\n' >> $c/button/component.rs"
mutate "undeclared effects class" "printf 'const _C: &str = \"dx-shimmer\";\n' >> $c/button/component.rs"
mutate "undeclared primitives feature" "printf 'use dioxus_primitives::calendar::DateRange;\n' >> $c/button/component.rs"
mutate "wrong crate version" "sed -i 's|=0.1.0|=0.0.9|' $c/accordion/component.json"
mutate "dropped root member" "sed -i '0,/preview\\/src\\/components\\/tabs/{/preview\\/src\\/components\\/tabs/d}' component.json"
[ "$fail" -eq 0 ] || exit 1
echo "check-component-manifests: OK -- the generator's --check is green and 12 mutations each turn it red."
