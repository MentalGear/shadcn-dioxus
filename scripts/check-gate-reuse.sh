#!/usr/bin/env bash
# check-gate-reuse.sh -- the promises of cargo-gate reuse (backlog row 320), in a deterministic shell test.
#
# scripts/run-gates.sh stores a PASS of each cargo gate under a key made from scripts/cargo-inputs-hash.sh and reuses
# it for an identical key. A wrong "identical" would turn ALL GREEN into a lie, so this pins:
#   hash   - bytes the cargo gates read change the hash: .rs, Cargo.lock, a component docs.md, a primitives .ts,
#            a file an `include_str!` pulls in (even a .css one), an untracked new .rs, an added/removed/renamed
#            asset-only .css; bytes they cannot read do not: dev-docs, playwright, other scripts, an asset-only
#            .css body, an ignored file; the hash is stable across runs and independent of the checkout path.
#   store  - a miss, a hit, a different key missing, FAIL forgetting a stored PASS, a corrupt file ignored, the
#            key changing with a command line, the toolchain env and the scripts themselves.
#   guards on the real repo - every cargo-running gate in run-gates.sh is in CARGO_GATES (or reviewed as always-run),
#            and every Rust source that reads a file at run time via CARGO_MANIFEST_DIR is in the reviewed list
#            below: a new reader is red until its input is shown to be inside the hashed set.
# No network, no cargo build, no browser; a few seconds.
#
# Usage: scripts/check-gate-reuse.sh
set -uo pipefail
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"
# shellcheck source=scripts/gate-reuse.sh
. "$repo/scripts/gate-reuse.sh"

fail=0
ok()  { printf '  ok   %s\n' "$1"; }
bad() { printf '  FAIL %s\n' "$1" >&2; fail=1; }
same() { if [ "$2" = "$3" ]; then ok "$1"; else bad "$1 (expected equal: $2 vs $3)"; fi; }
differ() { if [ "$2" != "$3" ]; then ok "$1"; else bad "$1 (expected a different hash, got $2)"; fi; }

tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
fx="$tmp/fx"
hash() { "$fx/scripts/cargo-inputs-hash.sh" --root "$fx"; }

# --- a fixture repo with the shapes that matter ---------------------------------------------------------
mkdir -p "$fx/scripts" "$fx/primitives/src/ts" "$fx/preview/src/components/btn" "$fx/preview/assets" \
         "$fx/dev-docs/backlog" "$fx/playwright" "$fx/cli/src"
cp "$repo/scripts/cargo-inputs-hash.sh" "$repo/scripts/gate-reuse.sh" "$fx/scripts/"
cd "$fx"
git init -q . && git config user.email t@t && git config user.name t
printf 'target/\nignored.rs\n' > .gitignore
printf '[workspace]\n' > Cargo.toml
printf '# lock v1\n' > Cargo.lock
printf 'fn main() {}\n' > primitives/src/lib.rs
printf 'export const a = 1;\n' > primitives/src/ts/a.ts
printf 'pub const THEME: &str = include_str!("../assets/theme.css");\n' > preview/src/main.rs
printf 'body { color: red }\n' > preview/assets/theme.css
printf 'a { color: red }\n' > preview/src/components/btn/style.css
printf '## Usage\n' > preview/src/components/btn/docs.md
printf 'pub fn x() {}\n' > cli/src/lib.rs
printf '# note\n' > dev-docs/note.md
printf '# row\n' > dev-docs/backlog/001-a.md
printf 'test("x")\n' > playwright/a.spec.ts
printf '#!/bin/sh\necho hi\n' > scripts/other.sh
git add -A && git commit -q -m fixture
base="$(hash)"
echo "check-gate-reuse: inputs hash"
same "hash is stable across two runs" "$base" "$(hash)"

# independent of where the tree lives
cp -a "$fx" "$tmp/fx2"; same "hash is independent of the checkout path" "$base" "$("$tmp/fx2/scripts/cargo-inputs-hash.sh" --root "$tmp/fx2")"; rm -rf "$tmp/fx2"

# changes that must not matter
printf '# note v2\n' > dev-docs/note.md; printf '# row v2\n' > dev-docs/backlog/001-a.md
printf 'test("y")\n' > playwright/a.spec.ts; printf '#!/bin/sh\necho bye\n' > scripts/other.sh
printf 'a { color: blue }\n' > preview/src/components/btn/style.css
printf 'junk\n' > ignored.rs
same "docs, specs, other scripts, an asset-only .css body and an ignored file are not cargo inputs" "$base" "$(hash)"
git checkout -q -- . ; rm -f ignored.rs

# changes that must matter
chg() { # label, then a command that edits the tree; the tree is restored afterwards
  local label="$1"; shift
  ( "$@" ) >/dev/null 2>&1
  differ "$label" "$base" "$(hash)"
  git checkout -q -- . ; git clean -fdq -e ignored.rs
}
chg "an edited .rs"                          sh -c "printf 'fn main() { 1; }\n' > primitives/src/lib.rs"
chg "an edited Cargo.lock"                   sh -c "printf '# lock v2\n' > Cargo.lock"
chg "an edited component docs.md"            sh -c "printf '## Usage\nmore\n' > preview/src/components/btn/docs.md"
chg "an edited primitives .ts"               sh -c "printf 'export const a = 2;\n' > primitives/src/ts/a.ts"
chg "an edited css that an include_str! pulls in" sh -c "printf 'body { color: blue }\n' > preview/assets/theme.css"
chg "a new untracked .rs"                    sh -c "printf 'fn z() {}\n' > primitives/src/z.rs"
chg "an added asset-only .css"               sh -c "printf 'b{}\n' > preview/src/components/btn/extra.css"
chg "a removed asset-only .css"              rm preview/src/components/btn/style.css
chg "a renamed asset-only .css"              git mv preview/src/components/btn/style.css preview/src/components/btn/s2.css
chg "a deleted .rs"                          rm cli/src/lib.rs
# an include target OUTSIDE the cargo roots (dev-docs/motion.md in the real tree) is content-hashed
printf 'pub const M: &str = include_str!("../../dev-docs/note.md");\n' > cli/src/lib.rs; git add -A; git commit -q -m inc
base="$(hash)"
chg "an edited include target outside the cargo roots" sh -c "printf '# note v3\n' > dev-docs/note.md"
printf 'pub const M: &str = include_str!("../../dev-docs/gone.md");\n' > cli/src/lib.rs
differ "a missing include target is not silently skipped" "$base" "$(hash)"
git checkout -q -- .

echo "check-gate-reuse: result store"
GATE_REUSE_DIR="$tmp/store"; export GATE_REUSE_DIR
k1="$(gate_reuse_key "$fx" "clippy-default=cargo clippy" "test=cargo test")"
k1b="$(gate_reuse_key "$fx" "clippy-default=cargo clippy" "test=cargo test")"
same "the key is stable" "$k1" "$k1b"
differ "a changed command line is a different key" "$k1" "$(gate_reuse_key "$fx" "clippy-default=cargo clippy -D" "test=cargo test")"
printf 'fn main() { 2; }\n' > primitives/src/lib.rs
k2="$(gate_reuse_key "$fx" "clippy-default=cargo clippy" "test=cargo test")"
differ "a changed source is a different key" "$k1" "$k2"
git checkout -q -- .
differ_key="$(RUSTFLAGS="-C debuginfo=0" gate_reuse_key "$fx" "clippy-default=cargo clippy" "test=cargo test")"
differ "RUSTFLAGS is part of the key" "$k1" "$differ_key"
cp "$fx/scripts/gate-reuse.sh" "$tmp/gr.bak"; echo '# edit' >> "$fx/scripts/gate-reuse.sh"
differ "an edit of the reuse logic drops every stored result" "$k1" "$(gate_reuse_key "$fx" "clippy-default=cargo clippy" "test=cargo test")"
cp "$tmp/gr.bak" "$fx/scripts/gate-reuse.sh"

if gate_reuse_lookup "$k1" clippy-default >/dev/null; then bad "a miss before anything was stored"; else ok "a miss before anything was stored"; fi
gate_reuse_record "$k1" clippy-default "abc12345" "256.1"
got="$(gate_reuse_lookup "$k1" clippy-default)"; rc=$?
if [ $rc = 0 ] && [[ "$got" == abc12345$'\t'*$'\t'256.1 ]]; then ok "a stored PASS is found with its tree and seconds"; else bad "a stored PASS is found (rc=$rc got='$got')"; fi
if gate_reuse_lookup "$k2" clippy-default >/dev/null; then bad "another key must miss"; else ok "another key misses"; fi
if gate_reuse_lookup "$k1" test >/dev/null; then bad "another gate must miss"; else ok "another gate under the same key misses"; fi
gate_reuse_forget "$k1" clippy-default
if gate_reuse_lookup "$k1" clippy-default >/dev/null; then bad "a FAIL must forget the stored PASS"; else ok "forget removes the stored PASS"; fi
mkdir -p "$tmp/store/$k1"; printf 'FAIL\tx\ty\t1\tformat1\n' > "$tmp/store/$k1/test"
if gate_reuse_lookup "$k1" test >/dev/null; then bad "only PASS lines may be reused"; else ok "a non-PASS file is never reused"; fi
printf 'garbage' > "$tmp/store/$k1/test"
if gate_reuse_lookup "$k1" test >/dev/null; then bad "a corrupt file must not be reused"; else ok "a corrupt file is never reused"; fi
unset GATE_REUSE_DIR
cd "$repo"

echo "check-gate-reuse: guards on this repo"
cargo_gates_re="$(grep -m1 -oE "CARGO_GATES='[^']+'" scripts/run-gates.sh | sed -E "s/^CARGO_GATES='(.*)'$/\1/")"
[ -n "$cargo_gates_re" ] || bad "CARGO_GATES not found in run-gates.sh"
# gates that run cargo yet are deliberately never reused (reason in the same line)
NEVER_REUSED='fmt'        # under a second and reads files cargo-inputs-hash.sh does not hash (playwright/ .rs fixtures)
while IFS=$'\t' read -r name cmd; do
  case "$cmd" in
    cargo\ *|*" cargo "*|*"cargo "*)
      if [[ "$name" =~ $cargo_gates_re ]]; then continue; fi
      if [[ " $NEVER_REUSED " == *" $name "* ]]; then continue; fi
      bad "gate $name runs cargo but is neither in CARGO_GATES nor reviewed as never-reused: $cmd" ;;
  esac
done < <(scripts/run-gates.sh --list 2>/dev/null)
for must in check-primitives-features check-attr-spread-collision clippy-default clippy-web clippy-server test; do
  [[ "$must" =~ $cargo_gates_re ]] || bad "$must is not reusable but must be"
done
[[ "fmt" =~ $cargo_gates_re ]] && bad "fmt must not be in CARGO_GATES"
ok "every cargo-running gate is classified (reusable or reviewed never-reused)"

# Rust sources that read files at RUN time through CARGO_MANIFEST_DIR. Each reads only inside the hashed set
# (cli/new.rs: templates/; default_checked: primitives/src; the preview ones: preview/tests/shadcn, preview/src/components
# docs.md, dev-docs/motion.md via include_str). A new reader must be added here AFTER checking its input is hashed.
REVIEWED_READERS='cli/src/new.rs primitives/src/default_checked.rs preview/src/chart_tooltip_parity.rs preview/src/chart_parity.rs preview/src/polar_parity.rs preview/src/main.rs preview/src/motion_docs.rs'
readers="$(git ls-files -z '*.rs' | xargs -0 grep -lE 'CARGO_MANIFEST_DIR' 2>/dev/null | grep -E '^(primitives|cli|preview|test-harness|dioxus-attributes|scripts/attr-spread-collision-checker)/' | sort | tr '\n' ' ')"
for r in $readers; do
  [[ " $REVIEWED_READERS " == *" $r "* ]] || bad "$r reads files via CARGO_MANIFEST_DIR but is not in REVIEWED_READERS (scripts/check-gate-reuse.sh): is its input inside cargo-inputs-hash.sh's set?"
done
ok "every CARGO_MANIFEST_DIR reader is reviewed"

# the real set has the include targets and not the noise
list="$(scripts/cargo-inputs-hash.sh --list)"
grep -qP '^C\t\S+\tdev-docs/motion\.md$' <<<"$list"        || bad "dev-docs/motion.md (include_str! in preview) is not content-hashed"
grep -qP '^C\t\S+\tpreview/assets/main\.css$' <<<"$list"     || bad "preview/assets/main.css (include_str! in preview) is not content-hashed"
grep -qP '^C\t\S+\tCargo\.lock$' <<<"$list"                  || bad "Cargo.lock is not hashed"
grep -qP '^C\t\S+\tpreview/build\.rs$' <<<"$list"            || bad "preview/build.rs is not hashed"
grep -qP '^C\t\S+\tprimitives/src/ts/' <<<"$list"            || bad "primitives/src/ts is not hashed"
grep -qP '^C\t\S+\tpreview/src/components/[a-z_]+/docs\.md$' <<<"$list" || bad "component docs.md are not hashed"
grep -qP '^P\t-\tpreview/src/components/[a-z_]+/style\.css$' <<<"$list" || bad "component style.css is not path-only"
if grep -qP '\t(dev-docs/backlog|playwright/)' <<<"$list"; then bad "dev-docs/backlog or playwright/ leaked into the cargo inputs"; fi
[ "$fail" = 0 ] && ok "the real tree's input set has the include targets and none of the noise"

if [ "$fail" = 0 ]; then echo "check-gate-reuse: OK"; else echo "check-gate-reuse: FAILED" >&2; fi
exit "$fail"
