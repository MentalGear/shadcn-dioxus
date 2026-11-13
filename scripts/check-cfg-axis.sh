#!/usr/bin/env bash
#
# check-cfg-axis.sh
#
# Rendered markup, component structure and attribute choice split on the primitives' `native` Cargo feature (`#[cfg(feature =
# "native")]` is the Blitz arm, `#[cfg(not(feature = "native"))]` the browser-engine arm every browser, webview and SSG
# build ships), never on `target_family = "wasm"`, which the SSG server build answers wrongly and so prerenders markup the
# client does not hydrate. Three more rules keep the renderer axis in one place: no `feature = "web"` cfg under
# `primitives/src/` (that feature is inert), no renderer feature cfg in a styled component under
# `preview/src/components/` (read `dioxus_primitives::NATIVE_ARM` in expression position instead), and every app-facing
# manifest feature that enables `dioxus/native` also enables `dioxus-primitives/native`.
#
# `target_family = "wasm"` stays legitimate only for genuinely wasm-only execution internals, on the gate's allowlist.
# The renderer is a question the wasm client and a host fullstack SSG server build answer alike, while `target_family`
# is an execution-target question. A styled component's cfg on a feature name would, after `dx components add`, name the
# user's crate.
#
# What this guards against: splitting markup on `target_family = "wasm"`, so
# the fullstack SSG prerender (a host binary, not wasm) renders the plain
# native arm -- no `popover` attribute, no `<dialog>` elements -- while the
# wasm client then hydrates against that structurally different markup,
# breaking events page-wide on every hard-loaded page of the deployed site. `playwright/oracle/
# hydration-parity.spec.ts` is the oracle for the symptom;
# this script guards the cfg pattern behind it.
#
# `target_family = "wasm"` remains legitimate ONLY for genuinely
# wasm-only *execution* internals -- and the only one is `sleep.rs` (allowlisted below):
# `document::eval` itself is a cross-renderer Dioxus API that compiles and
# runs (inertly, off a real browser/webview) on every target, so every
# leaf hook (`use_popover_sync`, `use_dialog_open_driver`, etc., in
# `top_layer.rs`/`lib.rs`) gates on `not(feature = "native")` too, which `cargo check -p
# dioxus-primitives` (host) and `--target
# wasm32-unknown-unknown` both accept. Should a genuinely wasm-only
# execution primitive ever get added, gate it inside a hook's *body*, one
# level under an outer `#[cfg(not(feature = "native"))]` -- never at module/
# component granularity -- and add it to the allowlist below rather than
# reintroducing a markup-level `target_family` split.
#
# Usage: scripts/check-cfg-axis.sh [--self-test]
#   --self-test   run the gate over fixture trees in a temporary directory: a clean tree stays green, and each of
#                 a `target_family` cfg, a `feature = "web"` cfg in primitives, a renderer cfg in a styled component and an
#                 app-facing feature that enables `dioxus/native` without `dioxus-primitives/native` turns it red
# Exit 0: clean. Exit 1: a rule above was broken, with file:line detail on stderr.

set -euo pipefail

self="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"

# The self-test runs this same script over a fixture tree (CFG_AXIS_ROOT); a normal run reads the repository.
if [[ "${1:-}" == "--self-test" ]]; then
  [[ "$#" -eq 1 ]] || { echo "check-cfg-axis: --self-test takes no argument" >&2; exit 2; }
  tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
  fixture() { # dir: a clean tree that exercises every rule
    mkdir -p "$1/primitives/src" "$1/preview/src/components/button" "$1/templates/starter"
    printf '%s\n' '#[cfg(not(feature = "native"))]' 'fn browser() {}' '#[cfg(feature = "native")]' 'fn blitz() {}' \
      '// a comment may name feature = "web" in prose without a cfg.' > "$1/primitives/src/lib.rs"
    printf '%s\n' '#[cfg(not(target_family = "wasm"))]' 'fn host() {}' > "$1/primitives/src/sleep.rs"
    printf '%s\n' 'use dioxus_primitives::NATIVE_ARM;' 'fn c() -> bool { NATIVE_ARM }' > "$1/preview/src/components/button/component.rs"
    printf '%s\n' '[features]' 'web = ["dioxus/web"]' 'native = ["dioxus/native", "dioxus-primitives/native"]' > "$1/preview/Cargo.toml"
    printf '%s\n' '[features]' 'web = ["dioxus/web"]' 'desktop = ["dioxus/desktop"]' > "$1/templates/starter/Cargo.toml.liquid"
  }
  fail=0
  expect() { # label, expected exit (0 or 1), fixture dir
    local rc=0
    CFG_AXIS_ROOT="$3" bash "$self" >"$tmp/out" 2>&1 || rc=$?
    if [[ "$rc" != "$2" ]]; then
      echo "check-cfg-axis --self-test: $1: expected exit $2, got $rc" >&2; sed 's/^/   | /' "$tmp/out" >&2; fail=1
    fi
  }
  fixture "$tmp/clean";                                                                        expect "a clean tree" 0 "$tmp/clean"
  fixture "$tmp/m1"; printf '%s\n' '#[cfg(target_family = "wasm")]' 'fn w() {}' >> "$tmp/m1/primitives/src/lib.rs"
                                                                                               expect "a target_family cfg outside the allowlist" 1 "$tmp/m1"
  fixture "$tmp/m2"; printf '%s\n' '#[cfg(feature = "web")]' 'fn w() {}' >> "$tmp/m2/primitives/src/lib.rs"
                                                                                               expect "a feature = \"web\" cfg in primitives" 1 "$tmp/m2"
  fixture "$tmp/m3"; printf '%s\n' '#[cfg_attr(not(feature = "web"), expect(dead_code))]' 'fn w() {}' >> "$tmp/m3/primitives/src/lib.rs"
                                                                                               expect "a negated feature = \"web\" cfg_attr in primitives" 1 "$tmp/m3"
  fixture "$tmp/m4"; printf '%s\n' '#[cfg(feature = "native")]' 'fn n() {}' >> "$tmp/m4/preview/src/components/button/component.rs"
                                                                                               expect "a native cfg in a styled component" 1 "$tmp/m4"
  fixture "$tmp/m5"; printf '%s\n' 'fn n() { if cfg!(not(feature = "web")) {} }' >> "$tmp/m5/preview/src/components/button/component.rs"
                                                                                               expect "a web cfg! in a styled component" 1 "$tmp/m5"
  fixture "$tmp/m6"; printf '%s\n' 'native = ["dioxus/native"]' >> "$tmp/m6/preview/Cargo.toml"
  sed -i '/^native = \["dioxus\/native", "dioxus-primitives\/native"\]$/d' "$tmp/m6/preview/Cargo.toml"
                                                                                               expect "preview's native feature without dioxus-primitives/native" 1 "$tmp/m6"
  fixture "$tmp/m7"; printf '%s\n' 'mobile = ["dioxus/mobile"]' 'blitz = ["dioxus/native"]' >> "$tmp/m7/templates/starter/Cargo.toml.liquid"
                                                                                               expect "the starter's blitz feature without dioxus-primitives/native" 1 "$tmp/m7"
  fixture "$tmp/ok1"; printf '%s\n' 'mobile = ["dioxus/mobile"]' 'blitz = ["dioxus/native", "dioxus-primitives/native"]' >> "$tmp/ok1/templates/starter/Cargo.toml.liquid"
                                                                                               expect "the starter's blitz feature with dioxus-primitives/native" 0 "$tmp/ok1"
  if [[ "$fail" == 0 ]]; then echo "check-cfg-axis --self-test: OK (a clean tree is green; each of the 4 rules turns red on a planted violation)"; fi
  exit "$fail"
fi
[[ "$#" -eq 0 ]] || { echo "check-cfg-axis: unknown option $1" >&2; exit 2; }

cd "${CFG_AXIS_ROOT:-$(dirname "${BASH_SOURCE[0]}")/..}"

SRC_DIR="primitives/src"

# Files allowed to contain a `target_family = "wasm"` cfg predicate, should
# one ever legitimately need to appear (see the module doc above). Only `sleep.rs` is listed;
# the leaf execution modules carry none --
# `document::eval` needs no inner wasm-only guard.
# Kept as a real allowlist, not a hardcoded zero,
# so a single narrowly-scoped exception doesn't require rewriting this
# script's logic, only adding a path here.
ALLOWLIST=(
  # "primitives/src/top_layer/mod.rs"
  # "primitives/src/lib.rs"
  # "primitives/src/scroll_lock.rs"
  # Not markup: `tokio` (native) and `gloo-timers` (wasm) are target-gated dependencies in
  # primitives/Cargo.toml, so the two arms of `sleep` must split on the same predicate.
  "primitives/src/sleep.rs"
)

is_allowlisted() {
  local path="$1"
  for allowed in "${ALLOWLIST[@]+"${ALLOWLIST[@]}"}"; do
    if [[ "$path" == "$allowed" ]]; then
      return 0
    fi
  done
  return 1
}

# Match only real cfg predicates -- `#[cfg(target_family = "wasm")]`,
# `#[cfg(not(target_family = "wasm"))]`, and the runtime `cfg!(target_family
# = "wasm")` macro form -- never a doc-comment or string mention of the
# phrase `target_family = "wasm"` on its own (a comment may name the wrong axis
# in prose to explain why it is wrong).
MATCHES="$(grep -rnE 'cfg!?\(.*target_family' "$SRC_DIR" || true)"

VIOLATIONS=""
if [[ -n "$MATCHES" ]]; then
  while IFS= read -r line; do
    file="${line%%:*}"
    if ! is_allowlisted "$file"; then
      VIOLATIONS+="$line"$'\n'
    fi
  done <<< "$MATCHES"
fi

red=0
if [[ -n "$VIOLATIONS" ]]; then
  echo "check-cfg-axis: found target_family cfg predicate(s) outside the allowlist:" >&2
  echo "$VIOLATIONS" >&2
  echo >&2
  echo "Markup/structure/attribute cfgs must split on feature = \"native\", not" >&2
  echo "target_family = \"wasm\" -- see dev-docs/reference/platform-rationale.md" >&2
  echo "Caveat 1 for the axis rule and why target_family is the wrong axis." >&2
  red=1
fi

# The renderer-feature rules: primitives never read `web`, a styled component never reads a renderer feature, and the
# app-facing manifests tie `dioxus/native` to `dioxus-primitives/native`.
python3 -I - <<'PY' || red=1
import pathlib, re, sys, tomllib

def code_only(text):
    # Drop `//` comment text (prose may name a feature) but keep line breaks so line numbers stay right.
    return re.sub(r"//[^\n]*", "", text)

def hits(root, pattern):
    out = []
    rx = re.compile(pattern)
    for path in sorted(pathlib.Path(root).rglob("*.rs")):
        text = code_only(path.read_text(encoding="utf-8", errors="replace"))
        for m in rx.finditer(text):
            out.append(f"{path}:{text.count(chr(10), 0, m.start()) + 1}: {m.group(0)}")
    return out

problems = []
for h in hits("primitives/src", r'feature\s*=\s*"web"'):
    problems.append(("a `feature = \"web\"` cfg under primitives/src (the `web` feature is inert; the browser arm is "
                     "`not(feature = \"native\")`)", h))
for h in hits("preview/src/components", r'feature\s*=\s*"(?:native|web)"'):
    problems.append(("a renderer feature cfg in a styled component (after `dx components add` it names the user's "
                     "crate; read `dioxus_primitives::NATIVE_ARM` in expression position)", h))

for manifest in ("preview/Cargo.toml", "templates/starter/Cargo.toml.liquid"):
    feats = tomllib.loads(pathlib.Path(manifest).read_text(encoding="utf-8")).get("features", {})
    for name, members in feats.items():
        on = {m.replace("dioxus?/", "dioxus/") for m in members}
        if "dioxus/native" in on and "dioxus-primitives/native" not in on:
            problems.append(("a feature that enables `dioxus/native` without `dioxus-primitives/native` (the primitives'"
                             " compile-time assert fails the build when a Blitz app is built with it)",
                             f"{manifest}: feature `{name}`"))
if problems:
    print("check-cfg-axis: the renderer axis is broken:", file=sys.stderr)
    for why, where in problems:
        print(f"  {where}\n      {why}", file=sys.stderr)
    sys.exit(1)
PY

[[ "$red" == 0 ]] || exit 1
echo "check-cfg-axis: OK -- no target_family cfg outside the allowlist, no feature = \"web\" cfg in primitives, no renderer cfg in a styled component, every dioxus/native feature forwards dioxus-primitives/native."
