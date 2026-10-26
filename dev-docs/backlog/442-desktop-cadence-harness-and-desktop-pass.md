---
id: 442
title: "Desktop proof for the JS plan: a desktop-cadence web build, Playwright WebKit and Firefox, and a recorded desktop-pass stamp"
status: "BACKLOG. Filed 2026-10-26 from the row 434 step 3 adversarial board. The delay hook comes with lane…"
size: "M"
area: tooling
updated: 2026-10-26
---

# Row 442: desktop proof for the row 434 JavaScript plan

## Item

Every spec runs on Chromium on the web, where the bridge to Rust is microtask-fast, so no existing spec can see a desktop-only latency or ordering bug, and no spec can see an engine difference. Row 434's plan routes work through the gate (G), through Rust over the bridge (R) and through the platform (A); each of those has a desktop and an engine half that nothing proves. The step 3 adversarial board found that the planned slow-channel mode (`window.__dxBridgeDelayFrames = N`) delays only bridge messages: it misses the rAF-bound edit cadence of the desktop renderer, an eval that runs before its own render's edits, the cross-channel reorder, the blocking request per Dioxus event, and every engine difference ([root-and-timing finding 4](../research/board-2026-10-26/root-and-timing.md), [gates-and-proof finding 8](../research/board-2026-10-26/gates-and-proof.md)). Until this row lands, every desktop claim in row 434 and its lane reports reads **"web-verified; desktop inferred"**.

## Status / trigger

BACKLOG. Filed 2026-10-26 from the row 434 step 3 adversarial board. The delay hook comes with lane 3.0's transport seam; the rest is independent and can start at any time. Lanes 3.PL, 3.GS and the three last ports (3.MS, 3.WB, the legacy removal) are what it proves.

## Size

M

## Detail

Estimate: about 14 agent-hours, inferred; the real-desktop pass itself is a person's time.

1. **A desktop-cadence web build.** A harness-only workspace that patches `dioxus-web` with `[patch.crates-io]` (about 30 lines against 0.7.9) so the web renderer behaves like the desktop one in the ways that matter (the timing contract is section 1.4 of [`../research/sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md)):
   - edits are flushed in a rAF and awaited before the next render (the hook is the commented-out `wait_for_raf` at `dioxus-web-0.7.9/src/lib.rs:256-262`), at most one render per frame;
   - the `eval` body runs in a task, not synchronously at the call;
   - `MountedData` getters resolve after a task hop;
   - `dioxus.send` is delivered after a task hop with N frames plus jitter, in arrival order per channel and not across channels.
   Events stay synchronous, as on desktop. N and the jitter come from one measured desktop latency distribution, not by fiat. The stuck setting `N = never` ("the patched getter never resolves") replaces the planned stuck-channel mode and drives the gate fail-open and WCAG 2.1.2 specs. The first version's `__dxBridgeDelayFrames` stays as the cheap bridge-latency harness (`?dx-bridge-delay=N`, row 443), never called a desktop model.
2. **Engines.** Playwright WebKit and Firefox. `playwright/playwright.config.ts` defines the projects, but the three local configs (`suite.local`, `ssg.local`, `oracle.local`) have one `chromium` project and no deploy gate runs the others, and only `chromium-1194` is installed in the lane container; browser discovery goes through `playwright/sandbox-chromium.cjs` (`check-chromium-path`), so the other engines need the same treatment. The matrix, named in the row so a lane report can quote it: `A/G specs: chromium (CI), webkit, firefox (manual)`. The named subset: `top-layer`, `combobox-placement`, `select-scroll-buttons`, `tooltip-arrow`, `popover`, `date-picker`, `close-on-scroll`, plus probes for `position-area` shift and size, `overflow-anchor` (Safari 27) and the Firefox lesson of `anchor_fallback.rs:21-28`.
3. **A recorded desktop pass.** One WebKitGTK or real-desktop pass per batch (a `workflow_dispatch` job with xvfb and WebKitWebDriver on a runner that can install webkit2gtk, or a person's Mac or Windows machine). `dev-docs/desktop-pass.md` holds the checklist: the C5 measurement table (residue doc section 3.5), a smoke check that `prevent_default` reaches the native event (a Dioxus `onkeydown` that cancels Tab), the gate's first-frame and fail-open checks, and the keyboard-inset case. The result is a stamp `desktop-pass/<tree>.pass` with the machine, OS, webview version and the numbers, the way `release-suite/<tree>.pass` works for the suite (`dev-docs/lane-rules.md`, deploy step). **Rule:** rows and lane reports say "web-verified; desktop inferred" until a stamp exists for the tree, and a G or R mapping is never marked done on web evidence alone.
4. **Not covered:** WebView2 and WKWebView behaviour beyond what the real pass records; only a machine proves that.
5. **Gates and docs:** the stamp rule goes into the lane report line (CLAUDE.md and `lane-rules.md`, owned by lane R's follow-up), and `desktop-pass.md` into the `dev-docs/README.md` map.
