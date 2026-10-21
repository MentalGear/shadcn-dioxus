---
id: 150
title: "Home-page blocks registered their hooks on the parent scope: MasonryCard called component.call(()), so…"
status: "OPEN (only the candidate .call(()) gate remains): FIXED 2026-10-04, merged (b83640a, PR #84 /…"
size: "M"
area: docs
updated: 2026-10-20
---

# Row 150: Home-page blocks registered their hooks on the parent scope: MasonryCard called component.call(()), so…

## Item

Home-page blocks registered their hooks on the parent scope: `MasonryCard` called `component.call(())`, so `BlockPlayer` grew a new never-cleared interval every ~1.2 s

## Status / trigger

**OPEN (only the candidate `.call(())` gate remains):** **FIXED 2026-10-04, merged** (`b83640a`, PR #84 / `001a5f0`, SCROLL-FIX lane); the candidate `.call(())` gate in the notes stays OPEN (measure false positives first) **2026-10-20 sync: checked, the candidate `.call(())` gate still does not exist in `scripts/`; row stays open for it only.**

## Size

M

## Detail

Found by the scroll-jank investigation (`dev-docs/research/scroll-jank-2026-10-04.md` section 7.1; its section 3 had the mechanism wrong). `Callback::call` runs its closure with the scope it was CREATED in on the scope stack, so a block that calls hooks directly (`BlockPlayer`, `BlockColorPalette`, `BlockCommand`, `BlockComposer`) pushed them onto `WidgetMasonry`'s hook list; the hook index resets only when that scope renders, so every later re-render of the card allocated a FRESH set (new signal, memo, effect and, for the player, a new `setInterval`) beside the old unreachable one: 128 live intervals after ~100 s, 1,326 timer wakeups/s at 155 s, 387 layouts during one scroll vs 0-1. Visible side effects, probed: player readout stuck at 1:24, pause did nothing, composer typing lost, colour-palette arrow keys no-op. **Construction (covers the class):** blocks mount as real components (`let Block = entry.component; rsx! { Block {} }` as `MasonryCard`'s children), each owning a scope, with the rule in `MasonryCard`'s doc; plus `dioxus_primitives::interval::use_interval` (a task spawned in the caller's scope and dropped with it, no JS timer) and `scripts/check-uncleared-intervals.sh` (bans `setInterval`/`set_interval`/`Interval::new`/`spawn_forever` unless marked `// interval-ok: <reason>`). **Not gated, and this row says so:** `check-self-subscribing-effects.sh` would not have caught it (the effect never read the signals; the spawned task is excluded by design) and `check-hooks-in-closures.sh` sees a hook lexically inside another hook's closure, not a hook reached through the `.call(())` of a callback created in another scope. **Candidate gate, not built:** flag `.call(())` of a component `Callback<_, Element>` inside rsx in `preview/src`, allowlisting render props that provably call no hooks; a `Callback<_, Element>` render prop is legitimate and common and a hook inside one is invisible to grep, so measure the false-positive rate over `preview/src` before deciding between a gate and a doc rule. Runtime backstop: `oracle/tier2-html/scroll-main-thread.spec.ts` tests 1 and 2 (live intervals constant; readout advances and pause works).
