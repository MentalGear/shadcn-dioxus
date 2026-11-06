#!/usr/bin/env bash
#
# check-public-api.sh   (backlog row 266, part c)
#
# The public API of `dioxus-primitives` changes only on purpose. `primitives/public-api.txt` is a sorted text snapshot of
# every item a user of the crate can name (`pub` fns with signatures, structs with their `pub` fields and `#[props(..)]`
# attributes, enums with variants, traits, consts, type aliases, macros, `pub mod`s, re-exports, inherent methods and
# trait impls of the public types, each `cfg` kept as an annotation), produced from the source by the `syn`-based
# `public-api-snapshot` tool (scripts/attr-spread-collision-checker/src/bin/public-api-snapshot.rs; no nightly rustdoc
# JSON, no new dependency). The gate fails when the tree's snapshot differs from the committed one and prints the diff.
# Updating the file is the deliberate act: it shows up in the diff of the change, where a reviewer reads it as an API change.
# A private change (a body, a private item, a comment) leaves the snapshot alone.
#
# Fix a red run: if the API change is intended, run `CARGO_TARGET_DIR=/abs/dir scripts/check-public-api.sh --update` and
# commit `primitives/public-api.txt` with the change; if it is not, restore the signature, visibility or field.
#
# A cargo gate: it builds the tool, so it needs an absolute, isolated CARGO_TARGET_DIR (run-gates.sh sets it).
#
# Usage: CARGO_TARGET_DIR=/abs/dir scripts/check-public-api.sh [--update | --self-test]
#   --update      rewrite primitives/public-api.txt from the tree and exit 0
#   --self-test   plant a `pub fn` (the gate goes red with a one-line diff), a private fn (stays green), and run the
#                 tool twice (identical output), in a temporary copy of primitives/src
# Exit status: 0 = the snapshot matches, 1 = it differs (or a self-test expectation broke), 2 = bad usage or environment.
set -uo pipefail
cd "$(dirname "$0")/.."

mode=check
case "${1:-}" in
  "") ;;
  --update) mode=update ;;
  --self-test) mode=self-test ;;
  *) echo "check-public-api: unknown option $1" >&2; exit 2 ;;
esac
[ "$#" -le 1 ] || { echo "check-public-api: takes at most one option" >&2; exit 2; }
case "${CARGO_TARGET_DIR:-}" in
  /*) ;;
  *) echo "check-public-api: CARGO_TARGET_DIR must be set to an absolute, isolated dir" >&2; exit 2 ;;
esac
export CARGO_TARGET_DIR

SNAPSHOT="primitives/public-api.txt"
echo "check-public-api: building public-api-snapshot..." >&2
# Offline first (a warm container has every dependency); a fresh container retries online with the locked versions.
if ! cargo build --release --offline -p attr-spread-collision-checker --bin public-api-snapshot >&2; then
  cargo build --release -p attr-spread-collision-checker --bin public-api-snapshot >&2 \
    || { echo "check-public-api: could not build the snapshot tool, offline or online" >&2; exit 2; }
fi
BIN="$CARGO_TARGET_DIR/release/public-api-snapshot"
[ -x "$BIN" ] || { echo "check-public-api: built but $BIN is not there" >&2; exit 2; }

tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT

if [ "$mode" = self-test ]; then
  fail=0
  cp -r primitives/src "$tmp/src"
  "$BIN" --root "$tmp/src" > "$tmp/a.txt" || exit 2
  "$BIN" --root "$tmp/src" > "$tmp/b.txt" || exit 2
  cmp -s "$tmp/a.txt" "$tmp/b.txt" || { echo "self-test: two runs differ" >&2; fail=1; }
  cmp -s "$tmp/a.txt" "$SNAPSHOT" || { echo "self-test: the tree's snapshot differs from $SNAPSHOT" >&2; fail=1; }
  printf '\nfn private_probe() {}\npub(crate) fn crate_probe() {}\n' >> "$tmp/src/lib.rs"
  "$BIN" --root "$tmp/src" > "$tmp/c.txt" || exit 2
  cmp -s "$tmp/a.txt" "$tmp/c.txt" || { echo "self-test: a private change moved the snapshot" >&2; fail=1; }
  printf '\npub fn planted_probe(x: u8) -> u8 { x }\n' >> "$tmp/src/lib.rs"
  "$BIN" --root "$tmp/src" > "$tmp/d.txt" || exit 2
  diff "$tmp/a.txt" "$tmp/d.txt" > "$tmp/d.diff"
  if [ "$(grep -c '^[<>]' "$tmp/d.diff")" != 1 ] || ! grep -q '^> planted_probe  fn planted_probe(x: u8) -> u8$' "$tmp/d.diff"; then
    echo "self-test: a planted pub fn must add exactly one line:" >&2; cat "$tmp/d.diff" >&2; fail=1
  fi
  if [ "$fail" = 0 ]; then echo "check-public-api --self-test: OK (stable, private change green, planted pub fn is one line)"; fi
  exit "$fail"
fi

"$BIN" --root primitives/src > "$tmp/now.txt" || { echo "check-public-api: the snapshot tool failed" >&2; exit 2; }
if [ "$mode" = update ]; then
  cp "$tmp/now.txt" "$SNAPSHOT"
  echo "check-public-api: wrote $SNAPSHOT ($(grep -vc '^#' "$SNAPSHOT") items)"
  exit 0
fi
if [ ! -f "$SNAPSHOT" ]; then
  echo "check-public-api: $SNAPSHOT is missing; create it with: CARGO_TARGET_DIR=$CARGO_TARGET_DIR scripts/check-public-api.sh --update" >&2
  exit 1
fi
if diff -u "$SNAPSHOT" "$tmp/now.txt" > "$tmp/diff"; then
  echo "check-public-api: OK ($(grep -vc '^#' "$SNAPSHOT") public items)"
  exit 0
fi
echo "check-public-api: RED -- the public API of dioxus-primitives changed:" >&2
cat "$tmp/diff" >&2
echo "If the change is intended: CARGO_TARGET_DIR=$CARGO_TARGET_DIR scripts/check-public-api.sh --update   and commit $SNAPSHOT with it." >&2
echo "If not: restore the signature, visibility or field." >&2
exit 1
