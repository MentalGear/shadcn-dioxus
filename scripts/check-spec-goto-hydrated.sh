#!/usr/bin/env bash
#
# check-spec-goto-hydrated.sh  (backlog rows 164, 427)
#
# A spec says which KIND of page it navigates to, in code: one helper per kind (`gotoHydrated` in `playwright/hydration.ts`,
# the other two in `playwright/page-kinds.ts`), and a bare
# `.goto(` is the only thing this gate fails. The SSG serves complete markup before the wasm bundle has hydrated it, and
# Dioxus attaches its event delegation only when hydration walks the tree, so a click or hover issued right after a plain
# `page.goto` is DROPPED, not queued (row 109). The kinds: `gotoHydrated` (the interactive app: `networkidle`, then
# `html[data-hydrated="true"]`), `gotoPrerendered` (the app, but the spec must not wait for hydration: JavaScript off, wasm
# aborted, a first-paint or stylesheet oracle, a commit-time observer; it refuses `about:` and `file:`) and `gotoReference`
# (a page that is not the app: a vendored W3C reference page, a route-fulfilled host, a static fixture, `about:blank`; it
# refuses a URL under the app's origin). Each helper checks its own claim, so no `goto-ok:` marker exists any more.
#
# Why a gate and not a `page.goto` override in the shared fixture (`playwright/fixtures.ts`): an override reaches
# only the `page` fixture, so every `browser.newContext().newPage()` (the tests that need their own viewport,
# touch or JS-disabled context) would silently keep the racy call. The gate sees every `.goto(` in every spec file.
#
# Rule 1. In every `playwright/**/*.ts` (not `node_modules`, `fixtures/`, or `hydration.ts` / `page-kinds.ts` themselves,
# where the helpers live), comments stripped, a `.goto(` call whose argument is not the bare identifier `page` (that is a spec-local
# `subject.goto(page)` helper, not Playwright's) fails, whatever a comment says: navigate through `gotoHydrated`,
# `gotoPrerendered` or `gotoReference`. The three helpers must be exported by their files, and `--self-test` (also run
# by every default run) turns each helper into the bare call it replaced and requires red.
#
# Rule 2. A spec names a component page by its PATH route (`/component/<name>/`), never the legacy query form
# `/component/?name=<x>&`. Every component has a prerendered file at the path route; the legacy route renders a
# name-agnostic "Loading component..." shell and redirects client-side once mounted (`preview/src/site/demos.rs`
# `ComponentDemo`), so `html[data-hydrated]` is already true on the SHELL before the real page exists. It also
# drops every query parameter except `iframe` and `dark_mode` and builds client-created (not prerendered)
# charts. A spec that tests the redirect itself carries `legacy-route-ok: <reason>` on the line or the line
# above. NOT covered: `/component/block/?name=<x>&variant=<y>&`. The server prerenders
# `/component/block/<name>/<variant>/` for Block components only (sidebar), so on a static host the legacy
# block shell is the only entry to any other component's variant page (the path form is a 404 there);
# `gotoHydrated` waits for that redirect to land.
#
# Rule 3. A path route carries no query text in its `<name>` segment: `/component/popover&variant=overlay/` is a 404
# (rewriting the query form `?name=popover&variant=overlay&` by hand produces exactly that, and the query route
# silently drops the `variant`).
#
# Rule 4. Query text is never glued to a path route: `${url(name)}variant=${variant}&` and `${URL}dark_mode=true` (with
# `URL` = `/component/drawer/`) build `/component/popover/variant=modal&`, a 404 that never sets `data-hydrated` and so
# times the test out at 90 s (a migration of `?name=drawer&dark_mode=true` to the path routes can leave one). A
# `dark_mode`/`variant`/`iframe` key must follow a `?` or `&`.
#
# Usage: scripts/check-spec-goto-hydrated.sh [FILE ...]   (default: every spec under playwright/, then the self-test;
# --self-test runs only the self-test)
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
self="$repo_root/scripts/check-spec-goto-hydrated.sh"

HELPERS=(gotoHydrated gotoPrerendered gotoReference)

self_test() {
  local dir fails=0 n=0 h
  dir="$(mktemp -d)"
  trap 'rm -rf "$dir"' RETURN
  expect() {  # expect <0|1> <name> <body>
    local want="$1" name="$2" body="$3" rc=0
    printf '%s\n' "$body" >"$dir/$name.spec.ts"
    bash "$self" "$dir/$name.spec.ts" >/dev/null 2>&1 || rc=$?
    n=$((n + 1))
    if [ "$rc" -ne "$want" ]; then
      echo "check-spec-goto-hydrated --self-test: FAIL $name: exit $rc, wanted $want" >&2
      fails=$((fails + 1))
    fi
  }
  for h in "${HELPERS[@]}"; do
    expect 0 "helper-$h" "await $h(page, url);"
    expect 1 "bare-for-$h" "await page.goto(url);"
    expect 1 "marker-for-$h" $'// goto-ok: not the app\nawait page.goto(url);'
  done
  expect 1 bare-other-receiver "await p1.goto(\`\${BASE}/x/\`, { waitUntil: 'load' });"
  expect 1 bare-chained "await (await ctx.newPage()).goto(url);"
  expect 1 bare-same-line-marker "await page.goto(url); // goto-ok: static page"
  expect 0 spec-local-helper "await subject.goto(page);"
  expect 1 legacy-route "await gotoHydrated(page, \`\${BASE}/component/?name=x&\`);"
  expect 1 glued-query "await gotoHydrated(page, \`\${URL}dark_mode=true\`);"
  [ "$fails" -eq 0 ] || exit 1
  echo "check-spec-goto-hydrated --self-test: OK ($n fixtures; each helper is green and its bare .goto( replacement, with or without a goto-ok marker, is red)"
}

if [ "${1:-}" = "--self-test" ]; then self_test; exit $?; fi

python3 - "$@" <<'PYEOF'
import os
import re
import sys

explicit = sys.argv[1:]
SKIP_DIRS = {"node_modules", "fixtures"}
SKIP_FILES = {"hydration.ts", "page-kinds.ts"}

def collect():
    out = []
    for root, dirs, names in os.walk("playwright"):
        dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS)
        out += [os.path.join(root, n) for n in sorted(names) if n.endswith(".ts") and n not in SKIP_FILES]
    return out

files = explicit or collect()
if not files:
    print("check-spec-goto-hydrated: no specs found (run from a checkout)", file=sys.stderr)
    sys.exit(2)

COMMENT_BLOCK = re.compile(r"/\*.*?\*/", re.S)
COMMENT_LINE = re.compile(r"(?<![:\"'\w])//[^\n]*")

def blank(m):
    return re.sub(r"[^\n]", " ", m.group(0))

GOTO = re.compile(r"\.\s*goto\s*\(\s*(?!page\s*\))")
LEGACY = re.compile(r"/component/\?name=|[\"'`]name=[A-Za-z_$\{]")
BAD_PATH = re.compile(r"/component/(?!block/)[A-Za-z0-9_${}.]+[&=]")
GLUED_QUERY = re.compile(r"[}/](?:dark_mode|variant|iframe)=")
LEGACY_OK = re.compile(r"legacy-route-ok:\s*\S")

errors = []
n_goto = n_legacy = 0

if not explicit:
    for helper_file, helper in (("hydration.ts", "gotoHydrated"), ("page-kinds.ts", "gotoPrerendered"), ("page-kinds.ts", "gotoReference")):
        with open(os.path.join("playwright", helper_file), encoding="utf-8") as fh:
            if not re.search(r"export\s+async\s+function\s+%s\b" % helper, fh.read()):
                errors.append(f"playwright/{helper_file} does not export `{helper}`, which this gate names as a way to navigate")

for path in files:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    raw_lines = raw.split("\n")
    code = COMMENT_LINE.sub(blank, COMMENT_BLOCK.sub(blank, raw)).split("\n")
    for i, line in enumerate(code):
        near = raw_lines[i] + "\n" + (raw_lines[i - 1] if i > 0 else "")
        if GOTO.search(line):
            n_goto += 1
            errors.append(f"{path}:{i + 1}: `{line.strip()[:90]}` is a bare `.goto(`: name the kind of page with "
                          f"gotoHydrated, gotoPrerendered or gotoReference (a click or hover issued before hydration is dropped)")
        if BAD_PATH.search(line):
            errors.append(f"{path}:{i + 1}: `{line.strip()[:90]}` puts query text in a `/component/<name>/` path segment (a 404)")
        if GLUED_QUERY.search(line):
            errors.append(f"{path}:{i + 1}: `{line.strip()[:90]}` glues a query key to a path route without `?` or `&` (a 404 that "
                          f"never sets data-hydrated)")
        if LEGACY.search(line):
            n_legacy += 1
            if not LEGACY_OK.search(near):
                errors.append(f"{path}:{i + 1}: `{line.strip()[:90]}` uses the legacy `?name=` route, whose redirect shell "
                              f"already carries data-hydrated")

if errors:
    for e in errors:
        print(e)
    print(f"\ncheck-spec-goto-hydrated: {len(errors)} problem(s). Navigate with `await gotoHydrated(page, url)` "
          f"(playwright/hydration.ts) and name a component by its path route (`/component/<name>/`); a navigation that must "
          f"not wait for hydration is `gotoPrerendered`, one to a page that is not the app is `gotoReference`; a test of the "
          f"legacy redirect carries `// legacy-route-ok: <reason>`.",
          file=sys.stderr)
    sys.exit(1)

print(f"check-spec-goto-hydrated: OK ({len(files)} spec file(s): no bare `.goto(`; "
      f"{n_legacy} legacy-route reference(s), each marked)")
PYEOF

if [ "$#" -eq 0 ]; then self_test; fi
