#!/usr/bin/env bash
# Code graph explorer (backlog row 486): rust-analyzer SCIP index -> graph.json -> explorer.html.
#
#   CARGO_TARGET_DIR=<absolute isolated dir> scripts/codegraph/run.sh [--force] [--blame] [--out DIR]
#
# Output (git-ignored, default <repo>/target-codegraph/): index.scip, graph.json, comments.json, status.json, explorer.html,
# and one *.key file per product. Open explorer.html from disk; it needs no network.
# Each product has its own inputs and its own key, rebuilt only when its key changed:
#   index.key     sha256 of the working-tree content hash of every *.rs and Cargo.toml that
#                 `git ls-files -co --exclude-standard` lists, Cargo.lock and the rust-analyzer version, taken when the index
#                 was built. The index is built when missing or with --force; a later source edit does not rebuild it, the
#                 page shows it STALE (with the changed files) instead. index.files keeps the hashes it was built from.
#   graph.key     index.key + build_graph.py + every *.ts and *.css (the extractor's static scans)
#   comments.key  sha256 of the content hash of every *.rs, *.ts, *.css, *.html, *.js, *.mjs and *.txt + comment_metrics.py +
#                 build_comments.py + scripts/comment-scan.py. File text only: a source edit rebuilds the metrics at once,
#                 even while the index is stale.
#   page.key      graph.key + comments.key + explorer.template.html + make_explorer.py + the cytoscape bundle + status.json
#   blame.key     (only with --blame, or while a blame.json whose key still matches exists) comments.key + HEAD +
#                 build_blame.py. `git blame --line-porcelain` per tracked node file, cached by the file's blob hash in
#                 blame/; build_blame.py turns it into the mean and median age of comment lines and of code lines.
# status.json records the HEAD, time and key each part was built from; the page's status line shows it.
# --force re-indexes (and so rebuilds everything). The SCIP run holds the build lock (scripts/lane-lock.sh).
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
force=0; blame=0; out="$repo/target-codegraph"
while [ $# -gt 0 ]; do
  case "$1" in
    --force) force=1 ;;
    --blame) blame=1 ;;
    --out) out="${2:?--out needs a directory}"; shift ;;
    *) echo "usage: CARGO_TARGET_DIR=<abs dir> $0 [--force] [--blame] [--out DIR]" >&2; exit 2 ;;
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
rs_now="$out/rs.now"
{ hashes '*.rs' ':(glob)**/Cargo.toml'; git hash-object Cargo.lock | sed 's/$/ Cargo.lock/'; } > "$rs_now"
tree_key="$( { cat "$rs_now"; echo "$ra_version"; } | sha)"
head_sha="$(git rev-parse --short=8 HEAD)"
dirty=0; [ -z "$(git status --porcelain)" ] || dirty=1

stored() { cat "$out/$1.key" 2>/dev/null || true; }
status=""

# index: built only when missing or forced. A source edit does NOT rebuild it: the page marks it stale instead, and the
# comment metrics (file text only) and the page are rebuilt from the current tree.
lock_before="$(git hash-object Cargo.lock)"
if [ "$force" = 1 ] || [ -z "$(stored index)" ] || [ ! -s "$out/index.scip" ]; then
  s0=$(date +%s)
  # nothing on the cargo side is fetched here: rust-analyzer loads the workspace as `cargo check` does
  CARGO_TARGET_DIR="$tgt" CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true \
    "$repo/scripts/lane-lock.sh" --label codegraph-scip -- \
    rust-analyzer scip "$repo" --output "$out/index.scip.tmp" >"$out/scip.log" 2>&1 \
    || { tail -5 "$out/scip.log" >&2; die "rust-analyzer scip failed (log: $out/scip.log)"; }
  mv "$out/index.scip.tmp" "$out/index.scip"
  [ "$(git hash-object Cargo.lock)" = "$lock_before" ] || die "Cargo.lock changed during the SCIP run"
  echo "$tree_key" > "$out/index.key"
  cp "$rs_now" "$out/index.files"
  printf 'head=%s\ndirty=%s\nat=%s\nsecs=%s\n' "$head_sha" "$dirty" "$(date +%s)" "$(( $(date +%s) - s0 ))" > "$out/index.info"
  rm -f "$out/graph.key" "$out/comments.key" "$out/page.key"
  status="index REBUILT $(( $(date +%s) - s0 ))s"
elif [ "$(stored index)" != "$tree_key" ]; then
  status="index reused, STALE (Rust/Cargo sources changed since it was built; --force re-indexes)"
else
  status="index reused"
fi
index_key="$(stored index)"

# graph: the index as built + the extractor + the static scans it reads
graph_key="$( { echo "$index_key"; git hash-object "$here/build_graph.py"; hashes '*.ts' '*.css'; } | sha)"
if [ "$(stored graph)" != "$graph_key" ] || [ ! -s "$out/graph.json" ]; then
  python3 -I "$here/build_graph.py" "$out/index.scip" "$out/graph.json" >"$out/graph.log" || die "extractor failed"
  echo "$graph_key" > "$out/graph.key"; rm -f "$out/page.key"
  status="$status, graph rebuilt"
else
  status="$status, graph reused"
fi

# comment metrics: file text only, never the index
comments_key="$( { hashes '*.rs' '*.ts' '*.css' '*.html' '*.js' '*.mjs' '*.txt'; git hash-object "$here/comment_metrics.py" "$here/build_comments.py" "$repo/scripts/comment-scan.py"; } | sha)"
if [ "$(stored comments)" != "$comments_key" ] || [ ! -s "$out/comments.json" ]; then
  python3 -I "$here/build_comments.py" "$out/comments.json" >"$out/comments.log" || die "comment metrics failed"
  echo "$comments_key" > "$out/comments.key"; rm -f "$out/page.key"
  printf 'head=%s\ndirty=%s\nat=%s\n' "$head_sha" "$dirty" "$(date +%s)" > "$out/comments.info"
  status="$status, comments rebuilt"
else
  status="$status, comments reused"
fi

# comment age vs code age (N5): git blame is slow (about 2 minutes for the whole tree), so it runs only with --blame
blame_key="$( { echo "$comments_key"; git rev-parse HEAD; git hash-object "$here/build_blame.py"; } | sha)"
if [ "$blame" = 1 ] || { [ -s "$out/blame.json" ] && [ "$(stored blame)" = "$blame_key" ]; }; then
  if [ "$(stored blame)" != "$blame_key" ] || [ ! -s "$out/blame.json" ]; then
    mkdir -p "$out/blame"; : > "$out/blame.index"
    git ls-files -z -- '*.rs' '*.ts' '*.css' '*.html' '*.js' '*.mjs' | while IFS= read -r -d '' f; do
      [ -f "$f" ] || continue
      bh="$(git hash-object -- "$f")"
      [ -s "$out/blame/$bh" ] || git blame --line-porcelain -- "$f" 2>/dev/null | grep -E '^(author-time |[0-9a-f]{40} )' > "$out/blame/$bh" || true
      printf '%s %s\n' "$bh" "$f" >> "$out/blame.index"
    done
    python3 -I "$here/build_blame.py" "$out" "$out/blame.json" >"$out/blame.log" || die "blame metrics failed"
    echo "$blame_key" > "$out/blame.key"; rm -f "$out/page.key"
    printf 'head=%s\nat=%s\n' "$head_sha" "$(date +%s)" > "$out/blame.info"
    status="$status, blame rebuilt"
  else
    status="$status, blame reused"
  fi
else
  rm -f "$out/blame.json" "$out/blame.key" "$out/blame.info"
  echo '{"files":{}}' > "$out/blame.json"
  status="$status, blame off (--blame)"
fi

# status.json: what each part was built from, shown in the page's status line
python3 -I - "$out" "$tree_key" "$head_sha" "$dirty" "$ra_version" <<'PY'
import json, sys
out, tree_key, head, dirty, ra = sys.argv[1:6]
def info(name):
    d = {}
    try:
        for ln in open(f"{out}/{name}.info"):
            k, _, v = ln.strip().partition("=")
            d[k] = v
    except OSError:
        pass
    return d
def rd(name):
    try:
        return open(f"{out}/{name}").read().strip()
    except OSError:
        return ""
def files(name):
    r = {}
    for ln in rd(name).splitlines():
        h, _, p = ln.partition(" ")
        r[p] = h
    return r
old, new = files("index.files"), files("rs.now")
changed = sorted(p for p in set(old) | set(new) if old.get(p) != new.get(p))
ix, cm, bl = info("index"), info("comments"), info("blame")
status = {
    "tree": {"head": head, "dirty": dirty == "1", "key": tree_key[:12]},
    "index": {"key": rd("index.key")[:12], "head": ix.get("head", "?"), "dirty": ix.get("dirty") == "1", "at": int(ix.get("at") or 0),
              "secs": int(ix.get("secs") or 0), "ra": ra, "stale": rd("index.key") != tree_key, "nchanged": len(changed), "changed": changed[:40]},
    "comments": {"key": rd("comments.key")[:12], "head": cm.get("head", "?"), "dirty": cm.get("dirty") == "1", "at": int(cm.get("at") or 0)},
    "blame": {"built": bool(bl), "head": bl.get("head", ""), "at": int(bl.get("at") or 0)},
}
json.dump(status, open(f"{out}/status.json", "w"), separators=(",", ":"))
PY
rm -f "$rs_now"

page_key="$( { echo "$graph_key"; echo "$comments_key"; git hash-object "$here/explorer.template.html" "$here/make_explorer.py" "$cyto" "$out/status.json" "$out/blame.json"; } | sha)"
if [ "$(stored page)" != "$page_key" ] || [ ! -s "$out/explorer.html" ]; then
  python3 -I "$here/make_explorer.py" "$here/explorer.template.html" "$out/graph.json" "$out/comments.json" "$out/blame.json" "$out/status.json" "$cyto" "$out/explorer.html"
  echo "$page_key" > "$out/page.key"
  status="$status, page rebuilt"
else
  status="$status, page reused"
fi
echo "codegraph: $status; index key ${index_key:0:12}, tree key ${tree_key:0:12}; $(( $(date +%s) - t0 ))s; $out/explorer.html"
