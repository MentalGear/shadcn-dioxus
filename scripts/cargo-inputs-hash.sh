#!/usr/bin/env bash
# Hash of everything the cargo gates (check-primitives-features, check-attr-spread-collision, clippy x3, test, test-server)
# can read, so scripts/run-gates.sh can reuse their PASS for an identical hash (backlog row 320).
#
#   scripts/cargo-inputs-hash.sh [--root DIR] [--list]
#
# Prints a 40-hex hash. `--list` prints the (class, id, path) rows the hash is made of instead.
#
# What goes in -- every WORKING-TREE file (tracked, or untracked and not ignored; a deleted tracked file simply
# drops out, which changes the hash) that is:
#   1. CONTENT-hashed (a changed byte changes the hash):
#      - the cargo roots: primitives/ cli/ preview/ test-harness/ dioxus-attributes/
#        scripts/attr-spread-collision-checker/ templates/ -- every file in them (Rust, Cargo.toml, build.rs,
#        primitives/src/ts and js, docs.md/notes.md/component.json that preview/build.rs and the preview tests
#        read, preview/tests/shadcn/*.json fixtures, index.html, .ftl, ...) EXCEPT the asset kinds below;
#      - the workspace files: Cargo.toml, Cargo.lock, rust-toolchain[.toml], rustfmt.toml, clippy.toml, their dotted
#        forms, .cargo/*, and the root component.json (cli reads it);
#      - the cargo gates' own scripts and baselines: scripts/check-primitives-features.sh,
#        scripts/check-attr-spread-collision.sh and its two *.baseline.tsv;
#      - every file a Rust source pulls in with a literal `include_str!("..")` / `include_bytes!("..")`, resolved
#        against the including file, wherever it lives (dev-docs/motion.md, README.md, preview/assets/main.css,
#        theme-presets.css, ...). This is derived on every run, so a NEW include is covered the day it is written.
#   2. PATH-only (the name counts, the bytes do not): *.css *.svg *.png *.jpg *.jpeg *.gif *.webp *.ico *.woff
#      *.woff2 under the cargo roots that no include pulls in. These are only ever reached through the `asset!`
#      macro, which needs the file to exist but does not let its bytes decide compile, clippy or test results;
#      adding, deleting or renaming one still changes the hash. A CSS-only change is therefore NOT a cargo input.
#
# What stays out, on purpose: dev-docs/ (except an included file), playwright/, the other scripts/, labs/,
# preview/public and every ignored path.
#
# What this does NOT see -- scripts/check-gate-reuse.sh guards the known routes: a Rust source that reads a file
# at RUN time (`CARGO_MANIFEST_DIR` joins) must live in the reviewed list there, so a new reader is a red gate
# until someone decides whether its input is inside the set above.
set -euo pipefail

root=""
list=0
while [ $# -gt 0 ]; do
  case "$1" in
    --root) root="${2:?--root needs a DIR}"; shift 2 ;;
    --list) list=1; shift ;;
    -h|--help) sed -n '2,6p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "cargo-inputs-hash: unknown option $1" >&2; exit 2 ;;
  esac
done
root="${root:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
cd "$root"
export LC_ALL=C

CARGO_ROOTS=(primitives cli preview test-harness dioxus-attributes scripts/attr-spread-collision-checker templates)
WORKSPACE_FILES=(Cargo.toml Cargo.lock rust-toolchain rust-toolchain.toml rustfmt.toml .rustfmt.toml clippy.toml
  .clippy.toml component.json scripts/check-primitives-features.sh scripts/check-attr-spread-collision.sh
  scripts/check-attr-spread-collision.baseline.tsv scripts/check-attr-spread-collision.component-forward.baseline.tsv)
ASSET_EXT='css|svg|png|jpg|jpeg|gif|webp|ico|woff|woff2'

# The working tree's files: tracked and untracked-not-ignored (NUL separated, then one per line).
all="$(git ls-files -z --cached --others --exclude-standard | tr '\0' '\n')"
if printf '%s\n' "$all" | grep -q $'\r'; then echo "cargo-inputs-hash: carriage return in a path" >&2; exit 2; fi

in_roots() { # $1 = path
  local r; for r in "${CARGO_ROOTS[@]}"; do [[ "$1" == "$r"/* ]] && return 0; done
  return 1
}
is_workspace_file() {
  local f; for f in "${WORKSPACE_FILES[@]}"; do [ "$1" = "$f" ] && return 0; done
  [[ "$1" == .cargo/* ]]
}

# Includes: <including file dir>/<literal>, normalised, relative to the root.
includes="$(
  printf '%s\n' "$all" | grep -E '\.rs$' | while IFS= read -r f; do
    [ -f "$f" ] || continue
    in_roots "$f" || continue
    grep -ohE 'include_(str|bytes)!\(\s*"[^"]+"\s*\)' "$f" 2>/dev/null | sed -E 's/^include_(str|bytes)!\(\s*"([^"]+)"\s*\)$/\2/' | while IFS= read -r lit; do
      realpath -m --relative-to="$root" "$root/$(dirname "$f")/$lit"
    done
  done | sort -u || true
)"

rows="$(
  printf '%s\n' "$all" | sort -u | while IFS= read -r f; do
    [ -n "$f" ] && [ -f "$f" ] || continue
    if is_workspace_file "$f"; then printf 'C\t%s\n' "$f"; continue; fi
    in_roots "$f" || continue
    if [[ "$f" =~ \.($ASSET_EXT)$ ]] && ! grep -qxF -- "$f" <<<"$includes"; then
      printf 'P\t%s\n' "$f"
    else
      printf 'C\t%s\n' "$f"
    fi
  done
  # Include targets outside the cargo roots (dev-docs/motion.md): content, or a loud 'M' (missing) row.
  while IFS= read -r f; do
    [ -n "$f" ] || continue
    in_roots "$f" && continue
    if [ -f "$f" ]; then printf 'C\t%s\n' "$f"; else printf 'M\t%s\n' "$f"; fi
  done <<<"$includes"
  true
)"
rows="$(printf '%s\n' "$rows" | sort -u)"

# Blob ids of the content rows in one `git hash-object` call.
content_paths="$(printf '%s\n' "$rows" | awk -F'\t' '$1 == "C" { print $2 }')"
ids="$(printf '%s\n' "$content_paths" | git hash-object --stdin-paths)"
out="$(paste -d'\t' <(printf '%s\n' "$content_paths") <(printf '%s\n' "$ids") | awk -F'\t' '{ print "C\t" $2 "\t" $1 }'
       printf '%s\n' "$rows" | awk -F'\t' '$1 != "C" { print $1 "\t-\t" $2 }')"
out="$(printf '%s\n' "$out" | sort -t$'\t' -k3,3)"

if [ "$list" = 1 ]; then printf '%s\n' "$out"; exit 0; fi
printf '%s\n' "$out" | sha1sum | cut -c1-40
