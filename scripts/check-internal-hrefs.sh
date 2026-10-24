#!/usr/bin/env bash
#
# check-internal-hrefs.sh
#
# Internal links on the docs site must not escape the deploy base path (the Pages build is served below
# `/shadcn-dioxus/`): no root-absolute string literal as an `href:` in `preview/src/**/*.rs` (build it from the router
# prefix or use a `Link { to: Route::... }`), no legacy `/component/?name=<x>` link in `preview/src` (the canonical
# route is `/component/<x>/`), and no router `Link { .. }` in `preview/src` without a `// link-ok: <reason>` marker,
# or ever with `aria_current` (the router's own `aria-current` differs between server and browser, so navigation links
# use `NavLink`, row 126).
#
#   1. No root-absolute string literal as an `href:` attribute in
#      preview/src/**/*.rs (`href: "/docs"`). Build the value from the router
#      prefix (`router().prefix()`) or use a `Link { to: Route::... }`.
#      (docs.md code samples are exempt: they show users' code, not ours.)
#   2. No legacy `/component/?name=<x>` link anywhere in preview/src -- docs.md
#      or .rs code. It is only a client-side redirect shell; the canonical
#      route is `/component/<x>/`. Doc comments (`//`, `///`, `//!`) that
#      merely describe the legacy form are fine.
#
#   3. No router `Link { .. }` in preview/src without a `// link-ok: <reason>`
#      comment on the same or the preceding line, and none ever with
#      `aria_current`. `dioxus_router::Link` computes its own `aria-current` as
#      `href == router.full_route_string()`: our routes carry `?:dark_mode`, so
#      the server renders `aria-current="page"` and the browser, whose route
#      string has no `?`, drops it on hydration (every navbar/footer/sidebar
#      link, site-wide); passing `aria_current` yourself emits it twice (Link
#      writes its literal beside `..attributes`). Backlog row 126. Navigation
#      links use `NavLink` (preview/src/nav_link.rs), whose single
#      `aria-current` comes from a path comparison identical on both sides; a
#      raw `Link` is for an external target or a content link that can never
#      point at its own page, and says so in the marker. `document::Link`
#      (the `<link>` tag) is not a router Link and is not matched.
#
# Usage: scripts/check-internal-hrefs.sh
# Exit status: 0 when clean.

set -uo pipefail
cd "$(dirname "$0")/.."

fail=0

hits=$(grep -rnE 'href:[[:space:]]*(format!\()?"/' preview/src --include='*.rs' || true)
if [ -n "$hits" ]; then
  echo "check-internal-hrefs: root-absolute href literal(s) in preview/src (escape the deploy base path):" >&2
  echo "$hits" >&2
  fail=1
fi

legacy=$(grep -rnF '/component/?name=' preview/src --include='*.md' --include='*.rs' \
  | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//' || true)
if [ -n "$legacy" ]; then
  echo "check-internal-hrefs: legacy /component/?name= link(s) -- use /component/<name>/:" >&2
  echo "$legacy" >&2
  fail=1
fi

# 3. Raw router `Link { .. }`: needs a `link-ok:` marker; never `aria_current`.
# `nav_link.rs` is exempt (its doc comment names `Link {`); `//` lines are skipped.
raw_links=$(find preview/src -name '*.rs' ! -name nav_link.rs -print0 | sort -z | xargs -0 awk '
  FNR == 1 { depth = 0; inlink = 0; prev = "" }
  {
    line = $0
    code = line; sub(/^[[:space:]]*\/\/.*/, "", code)
    if (!inlink && code ~ /(^|[^A-Za-z0-9_:])Link[[:space:]]*\{/) {
      inlink = 1; depth = 0; start = FNR
      if (line !~ /link-ok:/ && prev !~ /link-ok:/) {
        printf "%s:%d: raw router `Link` without a `// link-ok: <reason>` marker (use NavLink)\n", FILENAME, FNR
      }
    }
    if (inlink) {
      if (code ~ /aria[_-]current/) {
        printf "%s:%d: `aria_current` on a router `Link` (opened at line %d)\n", FILENAME, FNR, start
      }
      n = split(code, ch, "")
      for (i = 1; i <= n; i++) {
        if (ch[i] == "{") depth++
        else if (ch[i] == "}") { depth--; if (depth <= 0) { inlink = 0; break } }
      }
    }
    prev = line
  }')
if [ -n "$raw_links" ]; then
  echo "check-internal-hrefs: router Link misuse (dev-docs/backlog.md row 126; see preview/src/nav_link.rs):" >&2
  echo "$raw_links" >&2
  fail=1
fi

# 4. The library renders no raw router `Link { .. }` (row 126): a shipped
# component's current-page mark goes through `primitives/src/nav_link.rs`.
lib_links=$(grep -rnE '(^|[^A-Za-z0-9_:])Link[[:space:]]*\{' primitives/src --include='*.rs' \
  | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//' | grep -v 'link-ok:' || true)
if [ -n "$lib_links" ]; then
  echo "check-internal-hrefs: raw router Link in primitives/src (use nav_link::NavLink, row 126):" >&2
  echo "$lib_links" >&2
  fail=1
fi

if [ "$fail" -ne 0 ]; then
  exit 1
fi
echo "check-internal-hrefs: OK -- no root-absolute href literals, legacy /component/?name= links, or unmarked/aria_current router Links."
