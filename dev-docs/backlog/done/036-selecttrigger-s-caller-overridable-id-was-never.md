---
id: 36
title: "SelectTrigger's caller-overridable id was never resynced into ctx.selectable.trigger_id, the shared context…"
status: "Landed 2026-09-04"
size: "S–M"
area: tests
updated: 2026-09-04
---

# Row 36: SelectTrigger's caller-overridable id was never resynced into ctx.selectable.trigger_id, the shared context…

## Item

**LANDED 2026-09-04, verified by execution (Chromium).** `SelectTrigger`'s caller-overridable `id` was never resynced into `ctx.selectable.trigger_id`, the shared context signal `SelectList`'s `aria-labelledby` (row 34) reads from — a caller who sets their own `id` on `SelectTrigger` (as `oracle/tier2-html/top-layer.spec.ts`'s fixture does, `id: "clip-select-trigger"`) got a rendered trigger id that diverged from the internally-generated one the listbox labelled itself by, so the reference dangled (axe `aria-input-field-name`, `oracle/tier2-html/top-layer.spec.ts`'s "Select listbox open" test, previously RED and documented in-line). Fixed by construction, all five at once: `SelectTrigger`/`DropdownMenuTrigger`/`ContextMenuTrigger`/`MenubarTrigger`/`NavbarTrigger` each gained a typed `id: ReadSignal<Option<String>>` prop and feed their shared `trigger_id` context signal into `use_id_or` as its `gen_id`, mirroring `DialogTitle`'s existing `use_id_or(ctx.dialog_labelledby, props.id)` — a closer precedent than this row's originally-proposed `content_id` two-signal shape. `NavbarTrigger` was not named in either this row or its 2026-09-03 extension but was folded in alongside row 41a's `NavbarContent` `aria-labelledby` work, since it is the identical construction. New oracle `top-layer.spec.ts` Rule 14 covers all five; **it passed in the Chromium run**, as did the previously-RED "Select listbox open" axe test in the same file — Chromium only, no non-Chromium confirmation yet (row 4)

## Status / trigger

Landed 2026-09-04

## Size

S–M

## Detail

Found 2026-09-03 building row 34's axe coverage: every component spec's own (un-overridden) `Select` was unaffected and stayed green; only a caller overriding `SelectTrigger`'s `id` hit this, but that is a plausible thing for a real consumer to do, not just this one fixture. **2026-09-03 extension** (code review of the axe round) had identified the identical, then-unverified-in-practice id-desync class in `ContextMenuTrigger`, `MenubarTrigger` and `DropdownMenuTrigger` — each rendered its `id` via the caller-wins `merge_attributes` pattern while its content's `aria-labelledby` (rows 25/34) read the internal `trigger_id` signal, never resynced to a caller override the way `content_id` is; no spec exercised it at the time (only the `top_layer` fixture's `Select` override did). All three are covered by this session's single fix alongside `SelectTrigger` and `NavbarTrigger`
