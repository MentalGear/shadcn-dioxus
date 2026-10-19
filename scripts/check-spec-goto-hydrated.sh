#!/usr/bin/env bash
#
# check-spec-goto-hydrated.sh  (backlog row 164)
#
# The SSG serves complete markup before the wasm bundle has hydrated it, and Dioxus attaches its event delegation
# only when hydration walks the tree: a click or hover a spec issues right after a plain `page.goto` is DROPPED,
# not queued (row 109 -- the flake class that carousel, tooltip and hover-card specs hit one at a time). The cure
# is one helper, `gotoHydrated` (`playwright/hydration.ts`: `networkidle`, then `html[data-hydrated="true"]`, a
# flag the app's root effect sets only after a client render has committed). Row 109 migrated the specs that had
# failed; ~430 bare `page.goto(` calls were left, each a latent instance of the same race.
#
# Why a gate and not a `page.goto` override in the shared fixture (`playwright/fixtures.ts`): an override reaches
# only the `page` fixture, so every `browser.newContext().newPage()` (the tests that need their own viewport,
# touch or JS-disabled context) would silently keep the racy call; and it would be implicit magic with no
# written reason on the deliberate exceptions (a first-paint oracle MUST NOT wait for hydration). The gate sees
# every `.goto(` in every spec file, and the exceptions say why in the source.
#
# Rule 1. In every `playwright/**/*.ts` (not `node_modules`, `fixtures/`, or `hydration.ts` itself), comments
# stripped, a `.goto(` call whose argument is not the bare identifier `page` (that is a spec-local
# `subject.goto(page)` helper, not Playwright's) fails unless the line, or the line directly above it, carries
# `goto-ok: <reason>` -- a navigation that must not wait for hydration (a first-paint or pre-hydration oracle, a
# JavaScript-disabled or wasm-blocked context, a static fixture page that never sets the flag).
#
# Rule 2. A spec names a component page by its PATH route (`/component/<name>/`), never the legacy query form
# `/component/?name=<x>&`. Every component has a prerendered file at the path route; the legacy route renders a
# name-agnostic "Loading component..." shell and redirects client-side once mounted (`preview/src/main.rs`
# `ComponentDemo`), so `html[data-hydrated]` is already true on the SHELL before the real page exists. It also
# drops every query parameter except `iframe` and `dark_mode` and builds client-created (not prerendered)
# charts. A spec that tests the redirect itself carries `legacy-route-ok: <reason>` on the line or the line
# above. NOT covered: `/component/block/?name=<x>&variant=<y>&`. The server prerenders
# `/component/block/<name>/<variant>/` for Block components only (sidebar), so on a static host the legacy
# block shell is the only entry to any other component's variant page (the path form is a 404 there);
# `gotoHydrated` waits for that redirect to land.
#
# Rule 3. A path route carries no query text in its `<name>` segment: `/component/popover&variant=overlay/` is a 404
# (a migration from the legacy `?name=popover&variant=overlay&` produced exactly that, and the legacy route had
# silently dropped the `variant`).
#
# Usage: scripts/check-spec-goto-hydrated.sh [FILE ...]   (default: every spec under playwright/)
# Exit status: 0 when clean.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import os
import re
import sys

explicit = sys.argv[1:]
SKIP_DIRS = {"node_modules", "fixtures"}
SKIP_FILES = {"hydration.ts"}

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
GOTO_OK = re.compile(r"goto-ok:\s*\S")
LEGACY_OK = re.compile(r"legacy-route-ok:\s*\S")

errors = []
n_goto = n_legacy = 0

for path in files:
    with open(path, encoding="utf-8") as fh:
        raw = fh.read()
    raw_lines = raw.split("\n")
    code = COMMENT_LINE.sub(blank, COMMENT_BLOCK.sub(blank, raw)).split("\n")
    for i, line in enumerate(code):
        near = raw_lines[i] + "\n" + (raw_lines[i - 1] if i > 0 else "")
        if GOTO.search(line):
            n_goto += 1
            if not GOTO_OK.search(near):
                errors.append(f"{path}:{i + 1}: `{line.strip()[:90]}` is a bare `.goto(`: a click or hover issued before "
                              f"hydration is dropped")
        if BAD_PATH.search(line):
            errors.append(f"{path}:{i + 1}: `{line.strip()[:90]}` puts query text in a `/component/<name>/` path segment (a 404)")
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
          f"not wait for hydration carries `// goto-ok: <reason>`, a test of the legacy redirect `// legacy-route-ok: <reason>`.",
          file=sys.stderr)
    sys.exit(1)

print(f"check-spec-goto-hydrated: OK ({len(files)} spec file(s): {n_goto} deliberate non-hydrated `.goto(`, each marked; "
      f"{n_legacy} legacy-route reference(s), each marked)")
PYEOF
