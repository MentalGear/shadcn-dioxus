#!/usr/bin/env bash
#
# check-as-child.sh
#
# Every `as` render prop is `Option<Callback<AsChild, Element>>` (row 395): the callback gets the
# component's merged attributes, its handler props included, AND the children given to the component
# (`dioxus_primitives::AsChild`), so the caller's element keeps both, as Radix `asChild` does. A render
# callback typed `Callback<Vec<Attribute>, Element>` gets the attributes alone, and the component's
# children vanish when the caller picks the element. So no Rust source and no component docs.md may
# declare that shape, or pass `as` a closure that takes a bare `Vec<Attribute>`.
#
# Usage: scripts/check-as-child.sh
# Exit status: 0 when clean, 1 on a violation.
set -uo pipefail
cd "$(dirname "$0")/.."
roots=(primitives/src preview/src templates labs cli/src)
hits=$(grep -rnE 'Callback<[[:space:]]*Vec<Attribute>[[:space:]]*,[[:space:]]*Element[[:space:]]*>|\b(r#as|as):[[:space:]]*(move[[:space:]]+)?\|[^|]*:[[:space:]]*Vec<Attribute>[[:space:]]*\|' \
  --include='*.rs' --include='docs.md' --include='notes.md' "${roots[@]}" 2>/dev/null)
if [ -n "$hits" ]; then
  echo "FAIL: an \`as\` render prop that drops the component's children (take \`AsChild\`, render \`{el.children}\`):" >&2
  echo "$hits" >&2
  exit 1
fi
echo "OK: every \`as\` render prop takes AsChild (attributes and children)"
