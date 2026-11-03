#!/usr/bin/env bash
# Code graph explorer (backlog row 486): rust-analyzer SCIP index -> graph.json -> explorer.html.
#
#   CARGO_TARGET_DIR=<absolute isolated dir> scripts/codegraph/run.sh [--force] [--out DIR]
#
# Output (git-ignored, default <repo>/target-codegraph/): index.scip, graph.json, explorer.html, and one
# *.key file per product. Open explorer.html from disk; it needs no network.
# Staleness: each product is rebuilt only when its key changed.
#   index.key  sha256 of the working-tree content hash of every *.rs and Cargo.toml that
#              `git ls-files -co --exclude-standard` lists (so an uncommitted edit counts), Cargo.lock,
#              and the rust-analyzer version
#   graph.key  index.key + build_graph.py + every tracked/untracked *.ts and *.css (the extractor's static scans)
#   page.key   graph.key + explorer.template.html + make_explorer.py + the cytoscape bundle
# --force re-indexes (and so rebuilds the rest). The SCIP run holds the build lock (scripts/lane-lock.sh).
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
force=0; out="$repo/target-codegraph"
while [ $# -gt 0 ]; do
  case "$1" in
    --force) force=1 ;;
    --out) out="${2:?--out needs a directory}"; shift ;;
    *) echo "usage: CARGO_TARGET_DIR=<abs dir> $0 [--force] [--out DIR]" >&2; exit 2 ;;
  esac
  shift
done

die() { echo "codegraph: $*" >&2; exit 1; }
t0=$(date +%s)

# S7: absolute, isolated CARGO_TARGET_DIR; never the repo-root target/
tgt="${CARGO_TARGET_DIR:-}"; tgt="${tgt%/}"
[ -n "$tgt" ] || die "CARGO_TARGET_DIR is not set (an absolute, isolated dir, e.g. /home/user/tgt/codegraph)"
case "$tgt" in /*) ;; *) die "CARGO_TARGET_DIR must be absolute, got '$tgt'" ;; esac
case "$tgt" in "$repo"|"$repo/target"|"$repo/target/"*) die "CARGO_TARGET_DIR must not be the repo-root target/" ;; esac
# S8: generated artefacts stay out of tracked files
case "$out" in /*) ;; *) die "--out must be absolute" ;; esac
cd "$repo"
if [ -n "$(git ls-files -- "${out#"$repo"/}" 2>/dev/null)" ]; then die "output dir holds tracked files: $out"; fi
if ! git check-ignore -q "$out/index.scip" 2>/dev/null; then die "output dir is not git-ignored: $out"; fi
mkdir -p "$out" "$tgt"

command -v rust-analyzer >/dev/null || die "rust-analyzer missing (rustup component add rust-analyzer)"
ra_version="$(rust-analyzer --version)"
cyto="$here/node_modules/cytoscape/dist/cytoscape.min.js"
if [ ! -f "$cyto" ]; then
  (cd "$here" && pnpm install --frozen-lockfile --reporter=silent) || die "pnpm install --frozen-lockfile failed in scripts/codegraph"
fi
[ -f "$cyto" ] || die "cytoscape bundle missing: $cyto"

sha() { sha256sum | cut -d' ' -f1; }
# content hash per listed path, existing files only, deterministic order
hashes() {
  local list; list="$(mktemp)"
  git ls-files -co --exclude-standard -z -- "$@" | sort -z | while IFS= read -r -d '' f; do
    [ -f "$f" ] && printf '%s\n' "$f"
  done > "$list"
  paste -d' ' <(xargs -d '\n' -r git hash-object -- < "$list") "$list"
  rm -f "$list"
}
index_key="$( { hashes '*.rs' ':(glob)**/Cargo.toml'; git hash-object Cargo.lock; echo "$ra_version"; } | sha)"
graph_key="$( { echo "$index_key"; git hash-object "$here/build_graph.py"; hashes '*.ts' '*.css'; } | sha)"
page_key="$( { echo "$graph_key"; git hash-object "$here/explorer.template.html" "$here/make_explorer.py" "$cyto"; } | sha)"

stored() { cat "$out/$1.key" 2>/dev/null || true; }
status=""

lock_before="$(git hash-object Cargo.lock)"
if [ "$force" = 1 ] || [ "$(stored index)" != "$index_key" ] || [ ! -s "$out/index.scip" ]; then
  s0=$(date +%s)
  # nothing on the cargo side is fetched here: rust-analyzer loads the workspace as `cargo check` does
  CARGO_TARGET_DIR="$tgt" CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true \
    "$repo/scripts/lane-lock.sh" --label codegraph-scip -- \
    rust-analyzer scip "$repo" --output "$out/index.scip.tmp" >"$out/scip.log" 2>&1 \
    || { tail -5 "$out/scip.log" >&2; die "rust-analyzer scip failed (log: $out/scip.log)"; }
  mv "$out/index.scip.tmp" "$out/index.scip"
  [ "$(git hash-object Cargo.lock)" = "$lock_before" ] || die "Cargo.lock changed during the SCIP run"
  echo "$index_key" > "$out/index.key"
  rm -f "$out/graph.key" "$out/page.key"
  status="index REBUILT $(( $(date +%s) - s0 ))s"
else
  status="index reused"
fi
if [ "$(stored graph)" != "$graph_key" ] || [ ! -s "$out/graph.json" ]; then
  python3 -I "$here/build_graph.py" "$out/index.scip" "$out/graph.json" >"$out/graph.log" || die "extractor failed"
  echo "$graph_key" > "$out/graph.key"; rm -f "$out/page.key"
  status="$status, graph rebuilt"
else
  status="$status, graph reused"
fi
if [ "$(stored page)" != "$page_key" ] || [ ! -s "$out/explorer.html" ]; then
  python3 -I "$here/make_explorer.py" "$here/explorer.template.html" "$out/graph.json" "$cyto" "$out/explorer.html"
  echo "$page_key" > "$out/page.key"
  status="$status, page rebuilt"
else
  status="$status, page reused"
fi
echo "codegraph: $status; index key ${index_key:0:12}; $(( $(date +%s) - t0 ))s; $out/explorer.html"
