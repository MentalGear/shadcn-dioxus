#!/usr/bin/env bash
#
# check-cfg-axis.sh
#
# Rendered markup, component structure and attribute choice split on the `web` Cargo feature (`#[cfg(feature =
# "web")]` / `#[cfg(not(feature = "web"))]`), never on `target_family = "wasm"`: `web` is a renderer question, true
# for the wasm client and for a host fullstack SSG server build alike, while `target_family` is an execution-target
# question the SSG server build answers wrongly, so the prerendered markup would differ structurally from what the
# client hydrates. `target_family = "wasm"` stays legitimate only for genuinely wasm-only execution internals, inside
# a hook's body, on the gate's allowlist.
#
# What this guards against: splitting markup on `target_family = "wasm"`, so
# the fullstack SSG prerender (a host binary, not wasm) renders the plain
# native arm -- no `popover` attribute, no `<dialog>` elements -- while the
# wasm client then hydrates against that structurally different markup,
# breaking events page-wide on every hard-loaded page of the deployed site. `playwright/oracle/
# hydration-parity.spec.ts` is the regression oracle for the symptom;
# this script is the regression guard for the cfg pattern that caused it.
#
# `target_family = "wasm"` remains legitimate ONLY for genuinely
# wasm-only *execution* internals -- and none exist in this crate:
# `document::eval` itself is a cross-renderer Dioxus API that compiles and
# runs (inertly, off a real browser/webview) on every target, so every
# leaf hook (`use_popover_sync`, `use_dialog_open_driver`, etc., in
# `top_layer.rs`/`lib.rs`) gates on `feature = "web"` too, which `cargo check -p
# dioxus-primitives --features web` (host) and `--target
# wasm32-unknown-unknown` both accept. Should a genuinely wasm-only
# execution primitive ever get added, gate it inside a hook's *body*, one
# level under an outer `#[cfg(feature = "web")]` -- never at module/
# component granularity -- and add it to the allowlist below rather than
# reintroducing a markup-level `target_family` split.
#
# Usage: scripts/check-cfg-axis.sh
# Exit 0: clean. Exit 1: a `target_family` cfg predicate was found outside
# the allowlist, with file:line detail on stderr.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

SRC_DIR="primitives/src"

# Files allowed to contain a `target_family = "wasm"` cfg predicate, should
# one ever legitimately need to appear (see the module doc above). Empty:
# no occurrence remains, including in the leaf execution modules --
# `document::eval` needs no inner wasm-only guard.
# Kept as a real (if currently-empty) allowlist, not a hardcoded zero,
# so a single narrowly-scoped exception doesn't require rewriting this
# script's logic, only adding a path here.
ALLOWLIST=(
  # "primitives/src/top_layer.rs"
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

if [[ -n "$VIOLATIONS" ]]; then
  echo "check-cfg-axis: found target_family cfg predicate(s) outside the allowlist:" >&2
  echo "$VIOLATIONS" >&2
  echo >&2
  echo "Markup/structure/attribute cfgs must split on feature = \"web\", not" >&2
  echo "target_family = \"wasm\" -- see dev-docs/reference/platform-rationale.md" >&2
  echo "Caveat 1 for the corrected axis rule and the 2026-09-01 incident" >&2
  echo "that proved it." >&2
  exit 1
fi

echo "check-cfg-axis: OK -- no target_family cfg predicate outside the allowlist."
