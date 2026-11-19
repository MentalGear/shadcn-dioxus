#!/usr/bin/env bash
# Code graph explorer for this repo (row 486): runs MentalGear/code-analyser on this checkout with its project config
# `.codegraph/project.py` (the comment tokenizer scripts/comment-scan.py, the build lock, the component mapping,
# pairing and spec edges). Dev tool only, not a gate.
#
#   CARGO_TARGET_DIR=<absolute isolated dir> LANE_SLOTS=1 scripts/codegraph.sh [--force] [--blame] [--out DIR]
#   scripts/codegraph.sh smoke [path/to/explorer.html]     the tool's smoke.js on the built page
#
# The tool is found at $CODEGRAPH_HOME, else in a sibling checkout ../code-analyser. Output: target-codegraph/
# (git-ignored through target-*/); open target-codegraph/explorer.html from disk.
set -euo pipefail
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
home="${CODEGRAPH_HOME:-$(cd "$repo/.." && pwd)/code-analyser}"
if [ ! -x "$home/run.sh" ]; then
  echo "codegraph: the tool is not at $home" >&2
  echo "  get it: git clone https://github.com/MentalGear/code-analyser \"$home\"" >&2
  echo "  or point CODEGRAPH_HOME at a checkout of MentalGear/code-analyser" >&2
  exit 1
fi
if [ "${1:-}" = smoke ]; then
  shift
  exec node "$home/smoke.js" --repo "$repo" --project "$repo/.codegraph/project.py" "$@"
fi
exec "$home/run.sh" --repo "$repo" --project "$repo/.codegraph/project.py" "$@"
