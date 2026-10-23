#!/usr/bin/env bash
# Read-only dependency inventory for dev-docs/research/dependency-evaluation.md (row 371).
#   scripts/dep-report.sh [--network]
# Offline by default: per direct crate, the version in Cargo.lock, `unsafe` occurrences, build.rs / proc-macro
# presence in ~/.cargo/registry/src, and (through lane-cargo.sh, the build lock) the wasm and host transitive
# crate counts per primitives feature set. --network adds crates.io owners / last release / licence
# (generic User-Agent, no account data). Writes nothing in the tree.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$here"
net=0; [ "${1:-}" = "--network" ] && net=1
CRATES="${DEP_CRATES:-time time-macros palette palette_derive phf@0.11.3 rand@0.8.6 siphasher@1.0.3 approx by_address fast-srgb8 tracing serde tokio gloo-timers dioxus-icons pulldown-cmark pulldown-cmark-escape unicase}"
src=$(ls -d "$HOME"/.cargo/registry/src/*/ | head -1)
echo "## per crate (version in Cargo.lock; unsafe = lines containing the token in src/, comments included)"
printf '%-22s %-10s %7s %-9s %-10s\n' crate version unsafe build.rs proc-macro
for e in $CRATES; do
  c=${e%@*}; v=""; [ "$e" != "$c" ] && v=${e#*@}
  [ -n "$v" ] || v=$(awk -v n="$c" '$1=="name"&&$3=="\""n"\""{f=1;next} f&&$1=="version"{gsub(/"/,"",$3);print $3;exit}' Cargo.lock)
  d="$src$c-$v"; [ -d "$d" ] || { printf '%-22s %-10s (source not in registry cache)\n' "$c" "${v:-?}"; continue; }
  u=$(grep -rE '\bunsafe\b' "$d/src" 2>/dev/null | wc -l || true)
  b=no; [ -f "$d/build.rs" ] && b=yes
  p=no; { grep -qE '^\s*proc-macro\s*=\s*true' "$d/Cargo.toml" && p=yes; } || true
  printf '%-22s %-10s %7s %-9s %-10s\n' "$c" "$v" "$u" "$b" "$p"
done
echo; echo "## unique crates in the wasm32 normal tree of dioxus-primitives (deduped name+version)"
cnt(){ scripts/lane-cargo.sh -- cargo tree --offline -e normal --target wasm32-unknown-unknown --prefix none -p dioxus-primitives "$@" 2>/dev/null \
  | grep -v '^lane-' | sed -E 's/ \(\*\)//; s/ \(\/.*\)//' | sort -u | wc -l; }
echo "no features (router off):     $(cnt --no-default-features --features web)"
echo "+ calendar:                   $(cnt --no-default-features --features web,calendar)"
echo "+ color-picker:               $(cnt --no-default-features --features web,color-picker)"
echo "default (router on):          $(cnt --features web)"
echo; echo "## blast radius: components whose component.json asks for a crate or primitives feature"
for k in '"time"' '"palette"' '"dioxus-icons"' '"pulldown-cmark"' '"calendar"' '"color-picker"' '"router"'; do
  printf '%-16s %s\n' "$k" "$(grep -l "$k" preview/src/components/*/component.json | wc -l)"; done
echo "components total: $(ls preview/src/components/*/component.json | wc -l)"
if [ $net = 1 ]; then
  echo; echo "## crates.io (network)"
  for e in $CRATES; do c=${e%@*}
    curl -sS -m 30 -A shadcn-dioxus-research "https://crates.io/api/v1/crates/$c" | python3 -I -c 'import sys,json;d=json.load(sys.stdin);c=d["crate"];print(c["name"],c["max_version"],c["updated_at"][:10],"dl",c["downloads"])'
    curl -sS -m 30 -A shadcn-dioxus-research "https://crates.io/api/v1/crates/$c/owners" | python3 -I -c 'import sys,json;print("  owners:",[(u["login"],u["kind"]) for u in json.load(sys.stdin)["users"]])'
    sleep 0.5
  done
fi
