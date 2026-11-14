#!/usr/bin/env bash
#
# check-suite-reuse.sh  (backlog row 513)
#
# The release suite reuses a spec file's green result across a change that cannot affect it (playwright/tools/suite-key.mjs), and a
# wrong reuse would make a green deploy a lie, so this pins the rule on planted fixture repos and fake run reports, with no
# build, browser or suite run, then breaks each rule in a copy of the tools and requires the self-test to go red.
#
# What it pins: a spec-only change re-runs that file; a shared helper re-runs its importers; a site change, the config,
# fixtures.ts, the lock files or the environment re-run everything; dev-docs, docs and the tools re-run nothing, except the
# dev-docs file the build includes; a registered reader's data and an environment variable re-run the specs that read them; a
# record that is red, partial, hand-edited or for another build is never accepted; a chunk with run errors writes no record; a
# spec file with no record blocks the deploy; the verdict is the whole suite's (reused plus run). On the real tree every spec has
# a key, only the registered readers name their data, and no build input names an excluded path unreviewed.
#
# Usage: scripts/check-suite-reuse.sh        Exit status: 0 = OK; 1 = a promise broke (or a mutation stayed green).
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
command -v node >/dev/null || { echo "check-suite-reuse: node is required"; exit 1; }

out="$(node playwright/tools/suite-key-selftest.mjs 2>&1)" || { printf '%s\n' "$out"; echo "check-suite-reuse: FAIL -- the self-test is red"; exit 1; }
printf '%s\n' "$out" | tail -1

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fail=0
mutation() { # name file sed-expr -- breaks one rule in a copy of the tools; the fixtures-only self-test must go red
  local name="$1" file="$2" expr="$3" d="$tmp/$1"
  mkdir -p "$d/playwright"
  cp -r playwright/tools "$d/playwright/tools"
  sed -i -E "$expr" "$d/playwright/tools/$file"
  if cmp -s "playwright/tools/$file" "$d/playwright/tools/$file"; then
    echo "FAIL -- mutation '$name' did not change $file (the sed pattern is stale)"
  elif (cd "$d" && SUITE_KEY_SELFTEST_FIXTURES_ONLY=1 node playwright/tools/suite-key-selftest.mjs >/dev/null 2>&1); then
    echo "FAIL -- mutation '$name' stayed GREEN: the self-test does not guard that promise"
  else
    echo "ok -- mutation '$name' is red, as it must be"
  fi
}
names=(dev-docs-in-key include-not-pulled-back red-accepted partial-accepted chunk-errors-ignored import-walker-blind ambient-data-ignored env-ignored unkeyable-ignored public-sha-ignored)
mutation dev-docs-in-key suite-key.mjs 's|^export const EXCLUDED = \[.*|export const EXCLUDED = ["playwright/", "docs/", "primitives/ts-test/"];|' > "$tmp/${names[0]}.v" &
mutation include-not-pulled-back suite-key.mjs 's|const pulledBack = \[\.\.\.inc\.targets\]\.filter\(\(p\) => isExcluded\(p\)\)|const pulledBack = [...inc.targets].filter(() => false)|' > "$tmp/${names[1]}.v" &
mutation red-accepted suite-key.mjs 's/r\.status !== "green" \|\| //' > "$tmp/${names[2]}.v" &
mutation partial-accepted suite-key.mjs 's/ \|\| r\.partial !== false//' > "$tmp/${names[3]}.v" &
mutation chunk-errors-ignored suite-key.mjs 's/ \|\| \(j\.errors \?\? \[\]\)\.length\) \{ incomplete\.push\(!j/) { incomplete.push(!j/' > "$tmp/${names[4]}.v" &
wait
mutation import-walker-blind import-graph.mjs 's|^  /\\bfrom\\s\*\["'"'"'\]\(\[\^"'"'"'\\n\]\+\)\["'"'"'\]/g,|  /\\bfromX\\s*["'"'"']([^"'"'"'\\n]+)["'"'"']/g,|' > "$tmp/${names[5]}.v" &
mutation ambient-data-ignored suite-key.mjs 's/`ambient-data \$\{ambient\}`, //' > "$tmp/${names[6]}.v" &
mutation env-ignored suite-key.mjs 's/for \(const n of \[\.\.\.names\]\.sort\(\)\) rows\.push/for (const n of []) rows.push/' > "$tmp/${names[7]}.v" &
mutation unkeyable-ignored suite-key.mjs 's/if \(why\.length\) \{ out\.set/if (false) { out.set/' > "$tmp/${names[8]}.v" &
mutation public-sha-ignored suite-key.mjs 's/if \(publicSha && r\.publicSha !== publicSha\)/if (false)/' > "$tmp/${names[9]}.v" &
wait
for n in "${names[@]}"; do
  v="$(cat "$tmp/$n.v")"
  echo "$v"
  case "$v" in ok*) ;; *) fail=1 ;; esac
done
[ "$fail" = 0 ] || { echo "check-suite-reuse: FAIL"; exit 1; }
echo "check-suite-reuse: OK (self-test green, every mutation red)"
