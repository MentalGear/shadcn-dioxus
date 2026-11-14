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
names=(dev-docs-in-key include-not-pulled-back red-accepted proof-not-checked skipped-counts-as-green list-ignored skipped-recorded interrupted-recorded chunk-errors-ignored stale-report-read worktree-unchecked accept-guard-dropped import-walker-blind comments-stripped-again ambient-by-extension ambient-data-ignored env-ignored base-path-ignored unkeyable-ignored public-sha-ignored symlink-target-ignored)
mutation dev-docs-in-key suite-key.mjs 's|^export const EXCLUDED = \[.*|export const EXCLUDED = ["playwright/", "docs/", "primitives/ts-test/"];|' > "$tmp/dev-docs-in-key.v" &
mutation include-not-pulled-back suite-key.mjs 's|const pulledBack = \[\.\.\.inc\.targets\]\.filter\(\(p\) => isExcluded\(p\)\)|const pulledBack = [...inc.targets].filter(() => false)|' > "$tmp/include-not-pulled-back.v" &
mutation red-accepted suite-key.mjs 's/if \(r\.status !== "green"\) return/if (false) return/' > "$tmp/red-accepted.v" &
mutation proof-not-checked suite-key.mjs 's/if \(!r\.proof \|\| /if (false \&\& /' > "$tmp/proof-not-checked.v" &
mutation skipped-counts-as-green suite-key.mjs 's/c\.unexpected !== 0 \|\| c\.skipped !== 0\)/c.unexpected !== 0)/' > "$tmp/skipped-counts-as-green.v" &
wait
mutation list-ignored suite-key.mjs 's/if \(!proof\.equal\) \{ st\.equal = false;/if (false) { st.equal = false;/' > "$tmp/list-ignored.v" &
mutation skipped-recorded suite-key.mjs 's/if \(st\.skipped > 0\) \{ res/if (false) { res/' > "$tmp/skipped-recorded.v" &
mutation interrupted-recorded suite-key.mjs 's/if \(st\.irregular > 0\) \{ res/if (false) { res/' > "$tmp/interrupted-recorded.v" &
mutation chunk-errors-ignored suite-key.mjs 's/ \&\& !\(j\.errors \?\? \[\]\)\.length;/;/' > "$tmp/chunk-errors-ignored.v" &
mutation stale-report-read suite-key.mjs 's/Date\.parse\(j\.stats\.startTime\) >= startedAt/true/' > "$tmp/stale-report-read.v" &
wait
mutation worktree-unchecked suite-key.mjs 's/const problem = worktree\(pl\.tree\);/const problem = null;/' > "$tmp/worktree-unchecked.v" &
mutation accept-guard-dropped suite-key.mjs 's/if \(!\(passed > 0\)\) reasons\.push/if (false) reasons.push/' > "$tmp/accept-guard-dropped.v" &
mutation import-walker-blind import-graph.mjs 's/\(ts\.isImportDeclaration\(n\) \|\| ts\.isExportDeclaration\(n\)\)/(ts.isExportDeclaration(n))/' > "$tmp/import-walker-blind.v" &
mutation comments-stripped-again import-graph.mjs 's|^export function analyze\(text, file = "x\.ts"\) \{ |&text = text.replace(/\\/\\*[\\s\\S]*?\\*\\//g, ""); |' > "$tmp/comments-stripped-again.v" &
mutation ambient-by-extension suite-key.mjs 's/candidates\.filter\(\(p\) => !inSomeClosure\.has\(p\)/candidates.filter((p) => !MODULE_RE.test(p) \&\& !inSomeClosure.has(p)/' > "$tmp/ambient-by-extension.v" &
wait
mutation ambient-data-ignored suite-key.mjs 's/`ambient \$\{ambient\}`, //' > "$tmp/ambient-data-ignored.v" &
mutation env-ignored suite-key.mjs 's/for \(const n of \[\.\.\.names\]\.sort\(\)\) rows\.push/for (const n of []) rows.push/' > "$tmp/env-ignored.v" &
mutation base-path-ignored suite-key.mjs 's/path=\$\{basePathOf\(env\[n\]\)\}/path=/' > "$tmp/base-path-ignored.v" &
mutation unkeyable-ignored suite-key.mjs 's/if \(why\.length\) \{ out\.set/if (false) { out.set/' > "$tmp/unkeyable-ignored.v" &
mutation public-sha-ignored suite-key.mjs 's/if \(publicSha \&\& r\.publicSha !== publicSha\)/if (false)/' > "$tmp/public-sha-ignored.v" &
wait
mutation symlink-target-ignored suite-key.mjs 's/link \$\{fs\.readlinkSync\(p\)\} /link /' > "$tmp/symlink-target-ignored.v" &
wait
for n in "${names[@]}"; do
  v="$(cat "$tmp/$n.v")"
  echo "$v"
  case "$v" in ok*) ;; *) fail=1 ;; esac
done
[ "$fail" = 0 ] || { echo "check-suite-reuse: FAIL"; exit 1; }
echo "check-suite-reuse: OK (self-test green, every mutation red)"
