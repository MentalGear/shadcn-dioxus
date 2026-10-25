#!/usr/bin/env bash
#
# check-wrapper-props.sh
#
# A themed wrapper forwards every prop of the primitive it renders (row 391). The component docs page prints the
# primitive's props table next to the wrapper's (`preview/build.rs`, `preview/props_scan.rs`), so a prop the wrapper
# never passes is documented and does nothing: with no field of that name the value falls into the attribute spread.
# The scanner (`scripts/wrapper-props/main.rs`, sharing `preview/props_scan.rs`) finds every call of a primitive
# component in `preview/src/components/*/component.rs` (`module::Name { .. }`, a name imported from a primitive module,
# or a local component that takes a primitive's props struct) and lists each prop of that primitive the call does not
# pass by name or through a `..props` spread. Every listed prop must be forwarded or named, with its reason, in
# `scripts/check-wrapper-props.allow.tsv` (`path<TAB>module::Name.prop<TAB>reason`, or `*` for a whole fixture
# file). An allowlist entry that no longer matches a dropped prop fails too, so the list only shrinks.
#
# Usage: scripts/check-wrapper-props.sh
# Exit status: 0 when clean, 1 on a violation.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT
allow=scripts/check-wrapper-props.allow.tsv
rustc --edition 2021 -O -o "$out/wrapper-props" scripts/wrapper-props/main.rs 2>"$out/rustc.log" \
  || { cat "$out/rustc.log"; echo "check-wrapper-props: scanner failed to compile" >&2; exit 1; }
"$out/wrapper-props" primitives/src preview/src/components/*/component.rs | LC_ALL=C sort -u >"$out/dropped.tsv"

grep -v '^#' "$allow" | awk -F'\t' 'NF' >"$out/allow.tsv"
if awk -F'\t' 'NF < 3 || $3 == "" {print; f=1} END{exit !f}' "$out/allow.tsv" >"$out/bad.tsv"; then
  echo "check-wrapper-props: allowlist entries without a reason:" >&2; sed 's/^/  /' "$out/bad.tsv" >&2; exit 1
fi
awk -F'\t' '$2 == "*" {print $1}' "$out/allow.tsv" | LC_ALL=C sort -u >"$out/files.txt"
awk -F'\t' '$2 != "*" {print $1 "\t" $2}' "$out/allow.tsv" | LC_ALL=C sort -u >"$out/entries.tsv"
awk -F'\t' 'NR == FNR {skip[$1]; next} !($1 in skip)' "$out/files.txt" "$out/dropped.tsv" >"$out/checked.tsv"

rc=0
new="$(LC_ALL=C comm -23 "$out/checked.tsv" "$out/entries.tsv")"
stale="$(LC_ALL=C comm -13 "$out/checked.tsv" "$out/entries.tsv")"
if [ -n "$new" ]; then
  echo "check-wrapper-props: primitive props a themed wrapper does not pass (forward each one, or allowlist it with a reason in $allow):" >&2
  echo "$new" | sed 's/^/  /' >&2; rc=1
fi
if [ -n "$stale" ]; then
  echo "check-wrapper-props: stale allowlist entries (forwarded or gone: delete them from $allow):" >&2
  echo "$stale" | sed 's/^/  /' >&2; rc=1
fi
while read -r f; do
  [ -e "$f" ] || { echo "check-wrapper-props: allowlisted file $f does not exist" >&2; rc=1; }
done <"$out/files.txt"
[ "$rc" -eq 0 ] && echo "check-wrapper-props: OK (every primitive prop is forwarded or allowlisted; $(wc -l <"$out/entries.tsv") allowlisted)"
exit "$rc"
