#!/usr/bin/env bash
#
# check-suite-reuse.sh  (backlog row 513)
#
# The release suite reuses a spec file's green result across a change that cannot affect it (playwright/tools/suite-key.mjs), and a
# wrong reuse would make a green deploy a lie, so this pins the rule on planted fixture repos and fake run reports, with no
# build, browser or suite run, then breaks each rule in a copy of the tools and requires the self-test to go red.
#
# What it pins: a spec-only change re-runs that file; a change to anything else under playwright/ or primitives/ts-test/ (a
# helper, a tool, the config, data, a vendored script), to the site, the build environment, the browser or an environment
# variable a test reads re-runs everything; durations.json, dev-docs, docs and row files re-run nothing, except the dev-docs file
# the build includes; a `process` that could read the environment unseen, a test that names or enumerates spec files, a computed
# import and a missing TypeScript parser take the keys away; a record that is red, skipped, interrupted, unproved, stale,
# hand-edited, for another build or written after the worktree moved is never accepted; the verdict is the whole suite's. On the
# real tree every spec has a key and the reviewed sites are exactly today's.
#
# Usage: scripts/check-suite-reuse.sh        Exit status: 0 = OK; 1 = a promise broke (or a mutation stayed green).
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
command -v node >/dev/null || { echo "check-suite-reuse: node is required"; exit 1; }

node -e 'import("./playwright/tools/import-graph.mjs").then((m) => process.exit(/^typescript/.test(m.parserName()) ? 0 : 1))' \
  || { echo "check-suite-reuse: the TypeScript parser of scripts/ts-tools is missing (pnpm install --frozen-lockfile in scripts/ts-tools); the spec keys need it"; exit 1; }
out="$(node playwright/tools/suite-key-selftest.mjs 2>&1)" || { printf '%s\n' "$out"; echo "check-suite-reuse: FAIL -- the self-test is red"; exit 1; }
printf '%s\n' "$out" | tail -1

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fail=0
mutation() { # name file sed-expr -- breaks one rule in a copy of the tools; the fixtures-only self-test must go red
  local name="$1" file="$2" expr="$3" d="$tmp/$1"
  mkdir -p "$d/playwright" "$d/scripts"
  cp -r playwright/tools "$d/playwright/tools"
  ln -s "$repo_root/scripts/ts-tools" "$d/scripts/ts-tools"
  sed -i -E "$expr" "$d/playwright/tools/$file"
  if cmp -s "playwright/tools/$file" "$d/playwright/tools/$file"; then
    echo "FAIL -- mutation '$name' did not change $file (the sed pattern is stale)"
  elif (cd "$d" && SUITE_KEY_SELFTEST_FIXTURES_ONLY=1 node playwright/tools/suite-key-selftest.mjs >/dev/null 2>&1); then
    echo "FAIL -- mutation '$name' stayed GREEN: the self-test does not guard that promise"
  else
    echo "ok -- mutation '$name' is red, as it must be"
  fi
}
names=(dev-docs-in-key include-not-pulled-back red-accepted proof-not-checked skipped-counts-as-green list-ignored skipped-recorded interrupted-recorded undeclared-skip-accepted chunk-errors-ignored stale-report-read worktree-unchecked accept-guard-dropped shared-part-dropped tools-not-shared ts-test-not-shared durations-in-key process-sites-ignored spec-naming-ignored shared-site-ignored env-ignored base-path-ignored public-sha-ignored symlink-target-ignored)
mutation dev-docs-in-key suite-key.mjs 's|^export const EXCLUDED = \[.*|export const EXCLUDED = ["playwright/", "docs/", "primitives/ts-test/"];|' > "$tmp/dev-docs-in-key.v" &
mutation include-not-pulled-back suite-key.mjs 's|const pulledBack = \[\.\.\.inc\.targets\]\.filter\(\(p\) => isExcluded\(p\)\)|const pulledBack = [...inc.targets].filter(() => false)|' > "$tmp/include-not-pulled-back.v" &
mutation red-accepted suite-key.mjs 's/if \(r\.status !== "green"\) return/if (false) return/' > "$tmp/red-accepted.v" &
mutation proof-not-checked suite-key.mjs 's/if \(!r\.proof \|\| /if (false \&\& /' > "$tmp/proof-not-checked.v" &
mutation skipped-counts-as-green suite-key.mjs 's/c\.unexpected !== 0 \|\| c\.skipped !== 0\)/c.unexpected !== 0)/' > "$tmp/skipped-counts-as-green.v" &
mutation list-ignored suite-key.mjs 's/if \(!proof\.equal\) \{ st\.equal = false;/if (false) { st.equal = false;/' > "$tmp/list-ignored.v" &
wait
mutation skipped-recorded suite-key.mjs 's/if \(st\.skipped > 0\) \{ res/if (false) { res/' > "$tmp/skipped-recorded.v" &
mutation interrupted-recorded suite-key.mjs 's/if \(st\.irregular > 0\) \{ res/if (false) { res/' > "$tmp/interrupted-recorded.v" &
mutation undeclared-skip-accepted suite-key.mjs 's/t\.status === "skipped" \&\& t\.expectedStatus === "skipped" \&\& /t.status === "skipped" \&\& /' > "$tmp/undeclared-skip-accepted.v" &
mutation chunk-errors-ignored suite-key.mjs 's/ \&\& !\(j\.errors \?\? \[\]\)\.length;/;/' > "$tmp/chunk-errors-ignored.v" &
mutation stale-report-read suite-key.mjs 's/Date\.parse\(j\.stats\.startTime\) >= startedAt/true/' > "$tmp/stale-report-read.v" &
mutation worktree-unchecked suite-key.mjs 's/const problem = worktree\(pl\.tree\);/const problem = null;/' > "$tmp/worktree-unchecked.v" &
wait
mutation accept-guard-dropped suite-key.mjs 's/if \(!\(passed > 0\)\) reasons\.push/if (false) reasons.push/' > "$tmp/accept-guard-dropped.v" &
mutation shared-part-dropped suite-key.mjs 's/, `shared \$\{sharedHash\}`\]/]/' > "$tmp/shared-part-dropped.v" &
mutation tools-not-shared suite-key.mjs 's/!specPaths\.has\(p\) \&\& p !== "playwright\/durations\.json"/!specPaths.has(p) \&\& !p.startsWith("playwright\/tools\/") \&\& p !== "playwright\/durations.json"/' > "$tmp/tools-not-shared.v" &
mutation ts-test-not-shared suite-key.mjs 's/^const SHARED_ROOTS = \[.*/const SHARED_ROOTS = ["playwright\/"];/' > "$tmp/ts-test-not-shared.v" &
mutation durations-in-key suite-key.mjs 's/ \&\& p !== "playwright\/durations\.json"\)\.sort/).sort/' > "$tmp/durations-in-key.v" &
mutation process-sites-ignored suite-key.mjs 's/if \(blocking\) throw/if (false) throw/' > "$tmp/process-sites-ignored.v" &
wait
mutation spec-naming-ignored suite-key.mjs 's/const bad = sharedWhy \?\? own;/const bad = null;/' > "$tmp/spec-naming-ignored.v" &
mutation shared-site-ignored suite-key.mjs 's/const bad = sharedWhy \?\? own;/const bad = own;/' > "$tmp/shared-site-ignored.v" &
mutation env-ignored suite-key.mjs 's/for \(const n of \[\.\.\.names\]\.sort\(\)\) base\.push/for (const n of []) base.push/' > "$tmp/env-ignored.v" &
mutation base-path-ignored suite-key.mjs 's/path=\$\{basePathOf\(env\[n\]\)\}/path=/' > "$tmp/base-path-ignored.v" &
mutation public-sha-ignored suite-key.mjs 's/if \(publicSha \&\& r\.publicSha !== publicSha\)/if (false)/' > "$tmp/public-sha-ignored.v" &
mutation symlink-target-ignored suite-key.mjs 's/link \$\{fs\.readlinkSync\(p\)\} /link /' > "$tmp/symlink-target-ignored.v" &
wait
for n in "${names[@]}"; do
  v="$(cat "$tmp/$n.v")"
  echo "$v"
  case "$v" in ok*) ;; *) fail=1 ;; esac
done
[ "$fail" = 0 ] || { echo "check-suite-reuse: FAIL"; exit 1; }
echo "check-suite-reuse: OK (self-test green, every mutation red)"
