---
id: 78
title: "The native <dialog> un-freed-opposite-edge centering class"
status: "Landed 2026-09-18/19 (batch 2, Drawer + Sheet); swept clean elsewhere (batch 3)"
size: "S"
area: overlays
updated: 2026-09-18
---

# Row 78: The native <dialog> un-freed-opposite-edge centering class

## Item

The native `<dialog>` un-freed-opposite-edge centering class

## Status / trigger

Landed 2026-09-18/19 (batch 2, Drawer + Sheet); swept clean elsewhere (batch 3)

## Size

S

## Detail

**One construction, two instances this round, one clean sweep.** Both `Drawer` (`7310b89`) and `Sheet` (`12a8f86`) hit the identical bug shape independently: a per-side CSS rule overrides exactly ONE inset edge (e.g. `inset-inline-end: 0` on a right-anchored panel) while the UA's `dialog:modal` stylesheet's own `margin: auto` is left free to center the *opposite*, un-overridden edge — so the browser renders a floating, centered card instead of an edge-anchored panel. Measured on Sheet before the fix: a `[data-side="right"]` sheet's left edge at `x=440.5` against a 1280px-wide viewport (should sit flush right), and a `[data-side="bottom"]` sheet computing an 800px-tall box (should sit flush to the viewport's own height). Fixed identically in both: reset `margin: 0; inset: 0; width: auto; height: auto` as the shared base, then let each `data-side` variant re-open exactly the one edge it needs, so no UA default is ever left to fill a gap the component didn't ask for. **Swept for the same shape elsewhere, measured clean** — a concurrent batch-3 lane checked every other web-arm consumer of `primitives/src/dialog.rs` for the identical un-freed-opposite-edge shape (reported here as session scratchpad, not independently re-run by this docs pass): `Dialog` and `AlertDialog` both set `margin: 0` explicitly (killing the UA's auto-margin centering outright) and position via `top/left: <pct>; transform: translate(-50%, -50%)`, a self-contained technique with no dependency on the UA's un-set edges; `Command`'s command-palette dialog uses the same technique as `Dialog`; modal `Popover` uses `top_layer.rs`'s `ensure_anchor_positioning_styles`, which resets all four `inset` edges to `auto` at once, not one at a time. All four measured correctly centered/anchored at `:8080`, cross-checked against `getBoundingClientRect()` (every box's measured center matched its expected center exactly). One harmless, consistent artifact on all four: `getComputedStyle(...).right`/`.bottom` still report the dead UA-inherited value even though `getBoundingClientRect()` proves it unused in layout — an over-constrained `left`+`width`+`margin` box reporting a stale computed value, not a rendering bug, and not this class. **Subsumes:** Drawer, Sheet (both fixed by the shared construction above). **Does not subsume, because the shape never applied:** Dialog, AlertDialog, Command, modal Popover — swept and confirmed clean, a different centering technique entirely with no gap for this class to occur in
