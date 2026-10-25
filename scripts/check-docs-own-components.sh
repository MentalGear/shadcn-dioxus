#!/usr/bin/env bash
#
# check-docs-own-components.sh
#
# The docs site is built from the library's own components: no `details {` or `summary {` rsx element in the docs
# chrome sources and no `<details`/`<summary` emitted by `preview/build.rs`, which hands docs.md sections to the page
# as data. A hand-rolled disclosure is a second, unreviewed implementation of Accordion and Collapsible. Reasoned
# allowlist, stale entries fail; the browser half is `playwright/docs-accordion.spec.ts` and
# `playwright/docs-structure.spec.ts`.
#
# THE CLASS: a native `<details>`/`<summary>` disclosure for the component pages' sections (Installation, every
# docs.md `##` section, the Props reference, "Manual installation files", a demo's "About this example") drifts
# from the library's Accordion and Collapsible in look and motion (chevron overflowing the divider, nothing
# animating), and a fix to the components never reaches the docs. A docs page that hand-rolls a disclosure is a second,
# unreviewed implementation of a component the library already ships.
#
# THE CONSTRUCTION: the docs chrome uses `Accordion`/`AccordionItem`/`AccordionTrigger`/`AccordionContent` and
# `Collapsible` (`ComponentHighlight`, `VariantNotes` in preview/src/site/), and preview/build.rs emits docs.md
# sections as data (`render_docs_sections`), never as disclosure HTML. So this gate fails on:
#   - a `details {` or `summary {` rsx element in the docs chrome sources below (comment lines ignored);
#   - a `<details` or `<summary` emitted by preview/build.rs (comment lines ignored).
# The browser half is `playwright/docs-accordion.spec.ts` and `playwright/docs-structure.spec.ts`.
#
# ALLOWLIST: `<file>|<reason>` for a file that may keep a native disclosure (there is none today). An entry whose
# file with no hit fails (stale), and a reason is mandatory.
#
# Usage: scripts/check-docs-own-components.sh
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

RSX_FILES=(
  preview/src/main.rs
  preview/src/site/*.rs
  preview/src/docs_nav.rs
  preview/src/charts_gallery.rs
  preview/src/theme.rs
)
BUILD_RS=preview/build.rs

allowlist=()

# Non-comment lines of a Rust file, with their line numbers.
live_lines() {
  grep -nvE '^[[:space:]]*(//|/\*|\*)' "$1" || true
}

fail=0
declare -A hit=()
for f in "${RSX_FILES[@]}"; do
  [ -f "$f" ] || { echo "FAIL: $f is listed but missing (update RSX_FILES)"; fail=1; continue; }
  hits="$(live_lines "$f" | grep -E '(^|[^[:alnum:]_.-])(details|summary)[[:space:]]*\{' || true)"
  [ -n "$hits" ] && hit["$f"]="$hits"
done
if [ -f "$BUILD_RS" ]; then
  hits="$(live_lines "$BUILD_RS" | grep -E '<(details|summary)' || true)"
  [ -n "$hits" ] && hit["$BUILD_RS"]="$hits"
else
  echo "FAIL: $BUILD_RS is missing"; fail=1
fi

declare -A used=()
for f in "${!hit[@]}"; do
  allowed=0
  for entry in ${allowlist[@]+"${allowlist[@]}"}; do
    if [ "${entry%%|*}" = "$f" ]; then allowed=1; used["$f"]=1; fi
  done
  if [ $allowed = 0 ]; then
    echo "FAIL: $f renders a native <details>/<summary>; use the library's Accordion or Collapsible instead"
    head -3 <<<"${hit[$f]}" | sed 's/^/    /'
    fail=1
  fi
done

for entry in ${allowlist[@]+"${allowlist[@]}"}; do
  f="${entry%%|*}"; reason="${entry#*|}"
  [ -n "$reason" ] && [ "$reason" != "$entry" ] || { echo "FAIL: allowlist entry for $f has no reason"; fail=1; }
  [ -n "${used[$f]:-}" ] || { echo "FAIL: stale allowlist entry $f (no native disclosure left)"; fail=1; }
done

if [ $fail = 0 ]; then echo "check-docs-own-components: OK"; fi
exit $fail
