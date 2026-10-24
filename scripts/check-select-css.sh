#!/usr/bin/env bash
#
# check-select-css.sh   (backlog row 323)
#
# `playwright/tools/select.mjs` (pnpm run test:changed) answers an edit of `preview/assets/*.css` by reading the changed
# CSS RULES (not the file), placing the classes / ids / custom properties they touch on the pages of the SSG build
# under test, and selecting only the matrix tests whose page is reached; anything it cannot place falls back to the
# whole BROAD set. A selector that wrongly
# says "this rule cannot reach that page" is a hole in the test run, so the logic keeps its promises in a deterministic
# node test (`select.mjs --self-test`, playwright/tools/select-selftest.mjs): no browser, no build, no network.
#
#   1. the CSS parser and rule differ (comments, strings, at-rule contexts, an edit vs a MOVE of a rule);
#   2. the selector analysis (which classes a selector requires; :not()/attribute-only/element selectors require none);
#   3. page placement on a synthetic site (one page, shell-wide, closed-popup siblings, dead classes, tokens traced
#      to their readers, keyframes) and every fallback to the BROAD set;
#   4. the title shapes of the matrix specs against the real spec files (a spec whose titles stop naming pages fails
#      here instead of silently keeping every test);
#   5. the `--grep-invert` pattern drops exactly the dropped tests;
#   6. `--no-css-narrow` selects the whole BROAD set for any CSS edit.
#
# Then its own mutation proof: the same test is run against copies of the tools with one promise broken each time, and
# every copy must come out RED (a test that cannot fail guards nothing).
#
# Usage: scripts/check-select-css.sh        Exit status: 0 = OK; 1 = a promise broke (or a mutation stayed green).

set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

command -v node >/dev/null || { echo "check-select-css: node is required"; exit 1; }
[ -d playwright/node_modules ] || { echo "check-select-css: playwright/node_modules is missing (pnpm install --frozen-lockfile in playwright/)"; exit 1; }

out="$(node playwright/tools/select.mjs --self-test 2>&1)" || { printf '%s\n' "$out"; echo "check-select-css: FAIL -- the selector's self-test is red"; exit 1; }
printf '%s\n' "$out" | tail -1

# ---- mutation proof ------------------------------------------------------------------------------------------------
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkroot() { # a throwaway repo root: a copy of playwright/ (specs + tools), everything else linked
  local d="$1"
  mkdir -p "$d"
  tar --exclude=node_modules --exclude=test-results --exclude=playwright-report -cf - playwright | tar -xf - -C "$d"
  ln -s "$repo_root/playwright/node_modules" "$d/playwright/node_modules"
  for x in preview primitives dioxus-attributes .git; do [ -e "$repo_root/$x" ] && ln -s "$repo_root/$x" "$d/$x"; done
  return 0
}

# Each mutation runs as a background job and leaves THREE files: `<name>.verdict` (one line, written by the job),
# `<name>.err` (its stderr) and `<name>.status` (the job's exit status, written by a wrapper around it, so a job that is
# killed from outside, as memory pressure beside a full `run-gates.sh` can do, still has a status). `settle` turns those into a line: a job that exits without a verdict is reported as
# `FAIL -- mutation X produced no verdict (exit N): <stderr tail>`, never as a blank line.
# At most MUTATION_JOBS (default 2) run at once: each is a copy of playwright/ plus a node process, and four of them
# beside the gates' own load is the likely memory spike. A job that died without a verdict is retried once, alone, with
# a note, because losing a child to the machine says nothing about the promise under test; a second death is the FAIL.
MUTATION_JOBS="${MUTATION_JOBS:-2}"
case "$MUTATION_JOBS" in ''|*[!0-9]*|0) MUTATION_JOBS=2 ;; esac
names=()

mutation_job() { # name file sed-expr quick -- the body of one mutation; prints the verdict line
  local name="$1" file="$2" expr="$3" quick="$4" d="$tmp/$1"
  rm -rf "$d"
  mkroot "$d"
  sed -i -E "$expr" "$d/playwright/tools/$file"
  if cmp -s "$repo_root/playwright/tools/$file" "$d/playwright/tools/$file"; then
    echo "FAIL -- mutation '$name' did not change $file (the sed pattern is stale)"
  elif (cd "$d" && SELECT_SELFTEST_QUICK="$quick" node playwright/tools/select.mjs --self-test >/dev/null 2>&1); then
    echo "FAIL -- mutation '$name' stayed GREEN: the self-test does not guard that promise"
  else
    echo "ok -- mutation '$name' is red, as it must be"
  fi
}

launch() { # name file sed-expr quick -- starts the job in the background; the wrapper records its exit status
  local name="$1"
  rm -f "$tmp/$name.verdict" "$tmp/$name.err" "$tmp/$name.status"
  ( set +e; ( set -e; mutation_job "$@" ) >"$tmp/$name.verdict" 2>"$tmp/$name.err"; echo $? >"$tmp/$name.status" ) &
}

q_file=(); q_expr=(); q_quick=()
mutate() { # name file sed-expr [quick] -- queued; `run_mutations` starts them MUTATION_JOBS at a time
  names+=("$1"); q_file+=("$2"); q_expr+=("$3"); q_quick+=("${4:-0}")
}

run_mutations() {
  local i
  for i in "${!names[@]}"; do
    while [ "$(jobs -rp | wc -l)" -ge "$MUTATION_JOBS" ]; do wait -n 2>/dev/null || true; done
    launch "${names[$i]}" "${q_file[$i]}" "${q_expr[$i]}" "${q_quick[$i]}"
  done
  wait
}

settle() { # name -> prints the verdict line, or the reason there is none
  local name="$1" v="$tmp/$1.verdict" status tail
  status="$(cat "$tmp/$name.status" 2>/dev/null || true)"
  if [ -s "$v" ]; then cat "$v"; return; fi
  tail="$(tail -c 400 "$tmp/$name.err" 2>/dev/null | tr '\n' ' ')"
  case "$status" in
    "") status="unknown, the wrapper died too" ;;
    137) status="137 = SIGKILL, typically the OOM killer" ;;
    143) status="143 = SIGTERM" ;;
  esac
  echo "FAIL -- mutation '$name' produced no verdict (exit $status): ${tail:-no stderr}"
}

# the differ calls every edit "no change" -> a rule edit would select nothing
mutate differ-blind css-rules.mjs 's/return \{ removed, added, same: removed.length === 0 && added.length === 0 \};/return { removed: [], added: [], same: true };/' 1
# a selector requires no class or id -> every rule is "unconstrained" and the narrowing is never used
mutate selector-blind css-rules.mjs 's|^const IDENT = .*|const IDENT = String.raw`(?:\\x00)+`;|' 1
# a custom property is never traced to its readers -> a token change would reach no page
mutate token-blind css-impact.mjs 's/const reads = r.decls.some\(/const reads = false \&\& r.decls.some(/' 1
# every title that names no page is dropped -> an unknown title shape would lose tests
mutate drop-unscoped select.mjs 's/return !ks \|\| ks.length === 0 \|\| ks.some/return ks \&\& ks.some/'
run_mutations
bad=0
for i in "${!names[@]}"; do
  n="${names[$i]}"
  line="$(settle "$n")"
  case "$line" in
    "FAIL -- mutation '$n' produced no verdict"*)
      echo "check-select-css: note -- $line; retrying '$n' once, alone"
      launch "$n" "${q_file[$i]}" "${q_expr[$i]}" "${q_quick[$i]}"; wait
      line="$(settle "$n")" ;;
  esac
  echo "check-select-css: $line"
  case "$line" in ok*) ;; *) bad=1 ;; esac
done
[ "$bad" = 0 ] || exit 1

echo "check-select-css: OK -- self-test green, ${#names[@]} mutations red"
