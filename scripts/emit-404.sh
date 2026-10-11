#!/usr/bin/env bash
# Emits <public_dir>/404.html: the SPA fallback GitHub Pages serves (with status 404)
# for every URL the SSG did not prerender, so the app's router -- not GitHub's generic
# 404 page -- answers an unknown URL (dev-docs/backlog.md row 173).
#
# Usage: scripts/emit-404.sh <public_dir>
#
# Construction. The wasm client ALWAYS hydrates (dioxus-web's `hydrate` feature is on in
# a fullstack build; with no `window.initial_dioxus_hydration_data` it throws in `atob`,
# and with an empty `<div id="main">` the interpreter's hydrate pass throws), so the
# fallback has to be a real prerendered page. We take `demos/index.html`: it has the
# app's header + footer shell and, like the not-found routes, no docs sidebar. Every URL
# in it is base-path-absolute (`/shadcn-dioxus/assets/...`), so the page works at any
# depth. The browser location is not the page's own route, so the router renders the
# route for the URL actually requested (a component/chart not-found page, or the
# router's own no-match output) and the hydration pass takes over the shell's header
# and footer nodes; the demos `<main>` is replaced (measured: no stale nodes remain, no
# console errors, release build).
#
# That demos `<main>` would flash for one wasm load before being replaced, so the page
# hides `#main > main` with a style that a tiny head script removes as soon as the app
# sets `html[data-hydrated]` (preview/src/main.rs's App effect; the same signal the
# Playwright `gotoHydrated` waits for). Both live in <head>, outside the hydrated tree.
#
# The mismatch-free end state would be a dedicated prerendered not-found route
# (a router catch-all plus its entry in `server_static_routes`); until then this is the
# cheapest page that boots the app.
set -euo pipefail

public_dir="${1:?usage: scripts/emit-404.sh <public_dir>}"
src="$public_dir/demos/index.html"
out="$public_dir/404.html"

[ -f "$src" ] || { echo "error: $src not found -- nothing to derive 404.html from" >&2; exit 1; }

hide_css='<style id="dx-404-pending">#main>main{visibility:hidden}</style>'
reveal_js='<script>new MutationObserver(function(_,o){if(document.documentElement.dataset.hydrated){var s=document.getElementById("dx-404-pending");if(s)s.remove();o.disconnect()}}).observe(document.documentElement,{attributes:true,attributeFilter:["data-hydrated"]})</script>'

HIDE="$hide_css$reveal_js" perl -0777 -pe 's{</head>}{$ENV{HIDE}</head>}' "$src" > "$out"

# Verify the result is a bootable page with base-path-absolute assets.
fail() { echo "error: emit-404: $1" >&2; rm -f "$out"; exit 1; }
grep -q 'id="dx-404-pending"' "$out" || fail "the pending-content style was not injected"
grep -q 'initial_dioxus_hydration_data' "$out" || fail "no hydration data (the client throws without it)"
grep -q '<script type="module"' "$out" || fail "no wasm bootstrap <script type=\"module\">"
grep -q 'rel="stylesheet" href="/shadcn-dioxus/assets/' "$out" || fail "no base-path-absolute stylesheet"
grep -Eq '(href|src)="(\.|[A-Za-z0-9_-]+/)' "$out" && fail "a relative asset URL would break when served at depth"
echo "==> OK: $out ($(wc -c < "$out") bytes) is the app shell for unknown URLs."
