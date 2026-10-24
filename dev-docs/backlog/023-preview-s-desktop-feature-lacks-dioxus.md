---
id: 23
title: "preview's desktop feature lacks dioxus-primitives/web — a webview-backed desktop build would render the…"
status: "WAITING ON OWNER: a desktop build (needs gtk / webkit2gtk, absent in the sandbox). The fix is in…"
size: "S"
area: overlays
updated: 2026-10-24
---

# Row 23: preview's desktop feature lacks dioxus-primitives/web — a webview-backed desktop build would render the…

## Item

`preview`'s `desktop` feature lacks `dioxus-primitives/web` — a webview-backed desktop build would render the native (Blitz) markup arm even though `<dialog>`/`popover` work in a webview

## Status / trigger

**WAITING ON OWNER: a desktop build (needs gtk / webkit2gtk, absent in the sandbox).** The fix is in (3633b20, `dioxus-primitives/web` in preview's `desktop` feature) and verified at feature-resolution level only; no desktop build exists in the repo and none is exercised.

## Size

S

## Detail

**2026-09-26 (audit batch): FIXED** in `3633b20` -- `desktop = ["dioxus/desktop", "dioxus-primitives/web"]`, since Dioxus desktop renders in a system webview (a real browser engine), not Blitz. Verified at the feature-resolution level only (`cargo tree ... --features desktop -i dioxus-primitives` shows `web`); no desktop build exists in this repo yet. Same wiring gap the 2026-09-01 incident exposed for `server`; one-line fix, but untested here, so left for whoever first builds desktop
