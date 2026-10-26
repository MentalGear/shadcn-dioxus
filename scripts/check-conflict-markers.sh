#!/usr/bin/env bash
#
# check-conflict-markers.sh
#
# A merge once committed the conflict block that git writes into a file (the opener, the closer and the diff3 base
# line) into a Playwright spec, and no gate read the tracked files for it, so the file only failed when someone
# happened to run it. By construction, no tracked file may carry a line that starts with a conflict marker: the
# opener `<<<<<<<`, the closer `>>>>>>>` or the diff3 base `|||||||`, each followed by a space or the end of the line.
# A bare `=======` line is deliberately not flagged, because a Markdown setext heading underlines its title with
# exactly that line. The scan is `git grep` over the tracked files, so an untracked scratch file or a build output
# cannot fail it, and there is no allowlist: a conflict block is never intended text. The gate first runs its own
# self-test (fixtures generated at run time, so this script holds no marker line at column zero), so a broken
# pattern cannot turn the scan green.
#
# Usage: scripts/check-conflict-markers.sh [--self-test]
# Exit status: 0 when clean, 1 on a marker or a failed self-test, 2 on bad usage.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# The one pattern, shared by the tree scan and the self-test so the two cannot drift apart.
PATTERN='^(<<<<<<<|>>>>>>>)( |$)|^\|\|\|\|\|\|\| '

# Markers are built from variables so that no fixture line of this file starts with one.
open_m='<<<<<<<'
close_m='>>>>>>>'
base_m='|||||||'

failed=0

# check_fixture NAME red|green TEXT: the pattern must match TEXT (red) or must not (green).
check_fixture() {
  local name="$1" expect="$2" text="$3" hits
  hits="$(grep -nE "$PATTERN" <<<"$text" || true)"
  if [ "$expect" = red ] && [ -z "$hits" ]; then
    echo "check-conflict-markers: self-test: '$name' should be red but is green" >&2
    failed=1
  elif [ "$expect" = green ] && [ -n "$hits" ]; then
    echo "check-conflict-markers: self-test: '$name' should be green but is red: $hits" >&2
    failed=1
  fi
}

self_test() {
  failed=0
  check_fixture "opener and closer" red "ours
${open_m} HEAD
kept
${close_m} origin/main"
  check_fixture "diff3 base line" red "${base_m} base"
  check_fixture "bare opener" red "${open_m}"
  check_fixture "bare closer" red "${close_m}"
  check_fixture "markdown setext underline" green "Title
=======
body text"
  check_fixture "marker text mid-line" green "prose mentioning ${open_m} HEAD in the middle of a line"
  if [ "$failed" -ne 0 ]; then
    echo "check-conflict-markers: self-test FAILED" >&2
    return 1
  fi
  echo "check-conflict-markers: self-test OK (4 marker fixtures red, setext underline and mid-line text green)"
}

usage() { echo "usage: scripts/check-conflict-markers.sh [--self-test]" >&2; exit 2; }

if [ "$#" -gt 1 ]; then usage; fi
if [ "$#" -eq 1 ]; then
  [ "$1" = --self-test ] || usage
  if self_test; then exit 0; else exit 1; fi
fi

self_test || exit 1

rc=0
hits="$(git grep -nIE "$PATTERN" 2>&1)" || rc=$?
if [ "$rc" -gt 1 ]; then
  echo "FAIL: git grep could not scan the tracked files: $hits" >&2
  exit 1
fi
if [ "$rc" -eq 0 ]; then
  echo "FAIL: a tracked file carries a git conflict marker (resolve the conflict and commit the result):" >&2
  echo "$hits" >&2
  exit 1
fi
echo "OK: no tracked file carries a git conflict marker"
