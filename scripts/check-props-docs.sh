#!/usr/bin/env bash
# Every prop has a `///` doc comment, because every component page prints a Props reference table that
# `preview/build.rs` generates from the same scanner (`preview/props_scan.rs`); an undocumented prop is a table row
# with an empty description. Two shapes of prop, both scanned in `primitives/src/**` and every
# `preview/src/components/*/component.rs`, and both strict, with no baseline and no allowlist:
#   - a field of a `#[derive(Props)]` struct (pub or not: the builder setter is public either way);
#   - an argument of a `#[component] fn Name(a: A, ..)`.
# A `#[props(extends = ..)]` spread is not a prop (the table says "plus the global attributes") and is exempt; a
# component `docs.md` may not declare its own `## Props reference`.
#
# Every run first scans a fixture with one undocumented argument and one undocumented field and requires the scanner
# to name both, so a scanner that stops seeing a prop shape is red instead of silently clean.
#
# The scanner is std-only, so a bare `rustc` builds it in about a second; no cargo lock needed.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT
rustc --edition 2021 -O -o "$out/props-scan" scripts/props-scan/main.rs 2>"$out/rustc.log" \
  || { cat "$out/rustc.log"; echo "check-props-docs: scanner failed to compile" >&2; exit 1; }

# Self-test: the fixture has a documented argument (`ok`), an undocumented argument (`bare_arg`), an undocumented
# multi-line argument (`bare_multi`), a documented and an undocumented struct field; exactly the three bare ones must show.
cat >"$out/fixture.rs" <<'RS'
#[derive(Props, Clone, PartialEq)]
pub struct FixtureProps {
    /// Documented.
    pub ok: u8,
    pub bare_field: u8,
}

#[component]
pub fn Fixture(
    /// Documented.
    ok: u8,
    bare_arg: u8,
    #[props(default)]
    bare_multi: u8,
) -> Element {
    rsx! {}
}
RS
"$out/props-scan" "$out/fixture.rs" 2>/dev/null | awk -F'\t' '{print $1 " " $3}' | LC_ALL=C sort >"$out/fixture.got"
printf '%s\n' 'FN Fixture.bare_arg' 'FN Fixture.bare_multi' 'STRUCT FixtureProps.bare_field' >"$out/fixture.want"
cmp -s "$out/fixture.got" "$out/fixture.want" \
  || { echo "check-props-docs: self-test failed, the scanner reported:" >&2; cat "$out/fixture.got" >&2; exit 1; }

# Files split out of a `#[cfg(test)]` module (`tests.rs`, `*_tests.rs`, anything under a `tests/` directory) hold test components, not a public prop surface.
mapfile -t files < <({ find primitives/src -name '*.rs' ! -name tests.rs ! -name '*_tests.rs' ! -path '*/tests/*'; find preview/src/components -name component.rs; } | sort)
"$out/props-scan" "${files[@]}" >"$out/missing.tsv"

rc=0
if awk -F'\t' '$1=="STRUCT"{printf "%s:%s: %s has no doc comment\n", $2, $4, $3; f=1} END{exit !f}' "$out/missing.tsv"; then
  rc=1
fi
if awk -F'\t' '$1=="FN"{printf "%s:%s: #[component] fn argument %s has no doc comment\n", $2, $4, $3; f=1} END{exit !f}' "$out/missing.tsv"; then
  rc=1
fi
if [ "$rc" -eq 0 ]; then
  echo "check-props-docs: OK (every #[derive(Props)] field and #[component] fn argument is documented)"
else
  echo "check-props-docs: FAIL -- add a \`///\` comment above each prop (it becomes the Description column of the component's props table)" >&2
fi
exit "$rc"
