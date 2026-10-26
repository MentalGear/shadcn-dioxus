---
id: 442
title: "Desktop proof for the JS plan: a desktop-cadence web build, Playwright WebKit and Firefox, and a recorded desktop-pass stamp"
status: "BACKLOG. Filed 2026-10-26 from the row 434 step 3 adversarial board. The bridge-latency hook…"
size: "M"
area: tooling
updated: 2026-10-26
---

# Row 442: desktop proof for the row 434 JavaScript plan

## Item

Every spec runs on Chromium on the web, where the bridge to Rust is microtask-fast, so no existing spec can see a desktop-only latency or ordering bug, and no spec can see an engine difference. Row 434's plan routes work through the gate (G), through Rust over the bridge (R) and through the platform (A); each of those has a desktop and an engine half that nothing proves. The step 3 adversarial board found that the slow-channel mode (`window.__dxBridgeDelayFrames = N`, landed on `lane/js-bridge-3-0`) delays only bridge messages: it misses the rAF-bound edit cadence of the desktop renderer, an eval that runs before its own render's edits, the cross-channel reorder, the order in which the scheduler processes facts of different sessions, the blocking request per Dioxus event, and every engine difference ([root-and-timing finding 4](../research/board-2026-10-26/root-and-timing.md), [gates-and-proof finding 8](../research/board-2026-10-26/gates-and-proof.md)). Until this row lands, every desktop claim in row 434 and its lane reports reads **"web-verified; desktop inferred"**.

## Status / trigger

BACKLOG. Filed 2026-10-26 from the row 434 step 3 adversarial board. The bridge-latency hook (`__dxBridgeDelayFrames`, with the stuck setting `"never"`) is already on `lane/js-bridge-3-0`; the rest is independent and can start at any time. The witness harness (item 0.6 of the row 434 plan) and its order specs run first on this build; lanes 3.PL, 3.OV and the owner-measured ports (3.MS, 3.WB, the legacy removal) are what it proves.

## Size

M

## Detail

Estimate: about 14 agent-hours, inferred; the real-desktop pass itself is a person's time.

1. **A desktop-cadence web build.** A harness-only workspace that patches `dioxus-web` with `[patch.crates-io]` (about 30 lines against 0.7.9) so the web renderer behaves like the desktop one in the ways that matter (the timing contract is section 1.4 of [`../research/sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md)):
   - edits are flushed in a rAF and awaited before the next render (the hook is the commented-out `wait_for_raf` at `dioxus-web-0.7.9/src/lib.rs:256-262`), at most one render per frame;
   - the `eval` body runs in a task, not synchronously at the call;
   - `MountedData` getters resolve after a task hop;
   - `dioxus.send` is delivered after a task hop with N frames plus jitter, in arrival order per channel and not across channels;
   - facts of different sessions are processed in the scheduler's order, not in arrival order (`dioxus-core-0.7.9/src/scheduler.rs:106-110, 149-163, 247-258`), so an order spec for the root plan's order contract (epochs on facts, section 2.2) can see a fact overtaken by a later one.
   Events stay synchronous, as on desktop. N and the jitter come from one measured desktop latency distribution, not by fiat. The stuck setting `N = never` ("the patched getter never resolves") repeats the `"never"` of `__dxBridgeDelayFrames` on the desktop cadence and drives the gate fail-open and WCAG 2.1.2 specs there. `__dxBridgeDelayFrames` stays as the cheap bridge-latency harness (`?dx-bridge-delay=N`, row 443), never called a desktop model.
2. **Engines.** Playwright WebKit and Firefox. `playwright/playwright.config.ts` defines the projects, but the three local configs (`suite.local`, `ssg.local`, `oracle.local`) have one `chromium` project and no deploy gate runs the others, and only `chromium-1194` is installed in the lane container; browser discovery goes through `playwright/sandbox-chromium.cjs` (`check-chromium-path`), so the other engines need the same treatment. The matrix, named in the row so a lane report can quote it: `A/G specs: chromium (CI), webkit, firefox (manual)`. The named subset: `top-layer`, `combobox-placement`, `select-scroll-buttons`, `tooltip-arrow`, `popover`, `date-picker`, `close-on-scroll`, plus probes for `position-area` shift and size, `overflow-anchor` (Safari 27) and the Firefox lesson of `anchor_fallback.rs:21-28`.
3. **A recorded desktop pass.** One WebKitGTK or real-desktop pass per batch (a `workflow_dispatch` job with xvfb and WebKitWebDriver on a runner that can install webkit2gtk, or a person's Mac or Windows machine). `dev-docs/desktop-pass.md` (written by this row; it does not exist yet) holds the checklist: the C5 measurement table (residue doc section 3.5), a smoke check that `prevent_default` reaches the native event (a Dioxus `onkeydown` that cancels Tab), the gate's first-frame and fail-open checks, and the keyboard-inset case. The result is a stamp `desktop-pass/<tree>.pass` with the machine, OS, webview version and the numbers, the way `release-suite/<tree>.pass` works for the suite (`dev-docs/lane-rules.md`, deploy step). **Rule:** rows and lane reports say "web-verified; desktop inferred" until a stamp exists for the tree, and a G or R mapping is never marked done on web evidence alone.
4. **Not covered:** WebView2 and WKWebView behaviour beyond what the real pass records; only a machine proves that.
5. **Gates and docs:** the stamp rule goes into the lane report line (CLAUDE.md and `lane-rules.md`, which the docs lanes of row 434 do not edit: a main-loop follow-up), and `desktop-pass.md` into the `dev-docs/README.md` map.
6. **Round 5 additions** (board round 5; plan rows are listed in [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md)): the engines serve admission. A witness for an effect that a platform rule row covers runs on Playwright WebKit where it can observe the commit point (the clipboard write needs the user gesture there; a call three frames late still works on Chromium), and the `dragSource` half of `dropAnywhere` is judged on Firefox; witnesses for effects that serve another engine run under fallback emulation until this row's runs exist. The desktop-cadence build must also carry the order runtime's cases (a light dismiss followed by the trigger's own `onclick` in one gesture, which only the transition port makes agree on both renderers; text typed under async results; facts of two sessions in one batch). The latency session's prerequisites (a desktop smoke launch, a read-out that works on a release webview, a workload page) are item 0.5, [row 450](./450-phase-0-5-dxbridgestats-and-the-latency-session.md).
