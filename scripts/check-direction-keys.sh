#!/usr/bin/env bash
#
# check-direction-keys.sh
#
# Horizontal keyboard and gesture handling in `primitives/` is direction-aware by construction:
# `primitives/src/direction.rs` owns the one mirroring rule (`Direction::resolve_horizontal`, and `ltr_equivalent`
# built on it), so a non-test, non-comment line under `primitives/src` that names `Key::ArrowLeft`, `Key::ArrowRight`
# or `delta_x` puts its file on the hook: the file must also call `resolve_horizontal`, `ltr_equivalent` or
# `use_direction`, or sit on the reasoned ALLOWLIST (nothing to flip; a stale entry fails). Context structs hold the
# `ReadSignal<Direction>`, never a mount-time copy, so the RTL toggle reaches mounted descendants. Browser half:
# `playwright/oracle/tier3-radix/rtl.spec.ts` and `playwright/rtl-toggle.spec.ts`. Backlog row 137.
#
# THE CLASS: a handler that maps `Key::ArrowLeft`/`Key::ArrowRight` (or a physical horizontal pointer delta,
# `delta_x`) to "previous"/"next"/"decrease"/"increase" without asking the text direction is right in LTR and
# backwards in RTL, and nothing in the source tells it apart from a handler that mirrors.
#
# THE CONSTRUCTION: `primitives/src/direction.rs` owns the one mirroring rule (`Direction::resolve_horizontal`, and
# `Direction::ltr_equivalent`, which is built on it). A non-test, non-comment line under primitives/src that names
# `Key::ArrowLeft`, `Key::ArrowRight` or `delta_x` puts its FILE on the hook: the file must also call
# `resolve_horizontal`, `ltr_equivalent` or `use_direction` outside comments and tests (the file reads the direction),
# or be on the ALLOWLIST below with the reason there is nothing to flip. direction.rs itself is the rule.
# The browser half is `playwright/oracle/tier3-radix/rtl.spec.ts` and `playwright/rtl-toggle.spec.ts`.
#
# ALLOWLIST: `<file under primitives/src>|<reason>`. An entry that no longer matches a hook fails (stale), and a
# reason is mandatory.
#
# Usage: scripts/check-direction-keys.sh
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

allowlist=(
  "primitives/src/move_interaction.rs|produces PHYSICAL deltas (a pointer moving right is +x); its consumers mirror them: slider.rs and resizable.rs read use_direction, and color_picker.rs's saturation/lightness area is a physical 2-D plane"
  "primitives/src/color_picker.rs|the colour area and hue strip are physical planes (saturation grows to the right in every direction, as in native colour inputs): nothing to flip"
)

# stdin: a Rust file; prints the non-test, non-comment lines.
live_lines() {
  awk '/^[[:space:]]*#\[cfg\(test\)\]/ {exit} {print}' "$1" | grep -vE '^[[:space:]]*//' || true
}

fail=0
declare -A hit_used=()
while IFS= read -r f; do
  body="$(live_lines "$f")"
  grep -qE 'Key::Arrow(Left|Right)|delta_x' <<<"$body" || continue
  [ "$f" = primitives/src/direction.rs ] && continue
  if grep -qE 'resolve_horizontal|ltr_equivalent|use_direction' <<<"$body"; then continue; fi
  allowed=0
  for entry in "${allowlist[@]}"; do
    if [ "${entry%%|*}" = "$f" ]; then allowed=1; hit_used["$f"]=1; fi
  done
  if [ $allowed = 0 ]; then
    echo "FAIL: $f handles horizontal arrows/deltas without Direction::resolve_horizontal / ltr_equivalent / use_direction"
    grep -nE 'Key::Arrow(Left|Right)|delta_x' "$f" | grep -vE '^[0-9]+:[[:space:]]*//' | head -3 | sed 's/^/    /'
    fail=1
  fi
done < <(find primitives/src -name '*.rs' -not -path 'primitives/src/date_picker/tests/*' | sort)  # date_picker/tests/ is all test code (its parent declares it `#[cfg(test)] mod tests;`)

for entry in "${allowlist[@]}"; do
  f="${entry%%|*}"; reason="${entry#*|}"
  [ -n "$reason" ] && [ "$reason" != "$entry" ] || { echo "FAIL: allowlist entry for $f has no reason"; fail=1; }
  [ -n "${hit_used[$f]:-}" ] || { echo "FAIL: stale allowlist entry $f (no longer an undirected hit)"; fail=1; }
done

if [ $fail = 0 ]; then echo "check-direction-keys: OK"; fi
exit $fail
