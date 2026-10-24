---
id: 412
title: "Docs site: a Content-Security-Policy `<meta>` tag"
status: "OPTIONAL: needs owner approval before it is adopted or scheduled (owner 2026-10-24). Not part of…"
size: "S"
area: build
updated: 2026-10-24
---

# Row 412: Docs site: a Content-Security-Policy `<meta>` tag

## Item

Add a CSP to `preview/index.html` so that injected markup can't load remote scripts, plugins or a `<base>` override, and can't post forms off-site.

## Status / trigger

OPTIONAL: needs owner approval before it is adopted or scheduled (owner 2026-10-24). Not part of the adopted order. A lane must not pick it up until the owner says go.

## Size

S

## Detail

- GitHub Pages can't set response headers, so the policy has to go in a `<meta http-equiv="Content-Security-Policy">`. Browsers ignore `frame-ancestors`, `report-uri` and `sandbox` when they're set in a meta tag.
- **What limits the gain:** Dioxus 0.7.9's `document::eval` builds code with `js_sys::Function::new_with_args` (`dioxus-web-0.7.9/src/document.rs:220`), and primitives call eval in many places (`js_listener.rs`, `tooltip.rs`, `virtual_grid.rs`, `top_layer.rs` and others). So `script-src` needs `'unsafe-eval'`, which already covers WebAssembly compilation. A strict, eval-free script policy isn't possible without changing Dioxus.
- The two inline scripts in `index.html` (the theme and modality prepaint) need `sha256-` hashes. Add a gate that recomputes them so an edit can't silently break the page. Check also what inline script `dx` injects at build time.
- Proposed policy (to confirm in the spike): `default-src 'self'; script-src 'self' 'unsafe-eval' 'sha256-…' 'sha256-…'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'; form-action 'self'`. C13 already keeps remote assets out, so `'self'` should be enough.
- **Done when:** the full Playwright suite passes with the policy, the console shows no CSP violations on every route, and a gate checks the hashes. If the spike shows the policy can't be tighter than `'unsafe-inline'` scripts, report that to the owner instead of shipping it.

Source: `/mnt/project-files/backlog-drafts/2026-10-24-optional-safety-rows.md` (draft A), background in `/mnt/project-files/goose-eval/2026-10-24-goose-vs-wasm.md`.
