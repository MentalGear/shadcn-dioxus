---
id: 166
title: "HoverCard and Tooltip delay / close_delay props (Radix and Base UI parity)"
status: "LANDED 2026-10-15 (lane x166delays a49e875 + 9a98b55) -- recorded 2026-10-05 (audit); was scheduled…"
size: "S"
area: theme
updated: 2026-10-15
---

# Row 166: HoverCard and Tooltip delay / close_delay props (Radix and Base UI parity)

## Item

HoverCard and Tooltip `delay` / `close_delay` props (Radix and Base UI parity)

## Status / trigger

**LANDED 2026-10-15** (lane `x166delays` a49e875 + 9a98b55) -- recorded 2026-10-05 (audit); was scheduled in the next-session order (step 4, in row 138's batch: disjoint files)

## Size

S

## Detail

`primitives/src/hover_card.rs` (~`:57-60`) and `primitives/src/tooltip.rs` (~`:60-62`) already say in their module docs that delays are "a possible future prop" and that the component "opens and closes without one today": a hover card or tooltip opens the instant the pointer enters and closes the instant it leaves. The libraries underneath shadcn both have them: Radix `HoverCard` `openDelay`/`closeDelay`, Radix `Tooltip` `delayDuration`, Base UI `PreviewCard.Trigger` and `Tooltip.Trigger` `delay`/`closeDelay`. Without a delay, sweeping the pointer across a toolbar or a card list flashes every surface in turn, and a pointer travelling from the trigger to the content can close the card on the way. **Fix:** `delay` and `close_delay` (milliseconds) props on the two roots, defaults taken from the current Base UI source at build time (do not copy numbers from this row), timers owned by the scope and cleared on drop (the `check-uncleared-intervals.sh` class; a `setTimeout` through `document::eval` is a row 159 leak, so use the Rust-side timer the other components use), keyboard focus opening without the delay (check shadcn's tooltip), the two "possible future prop" paragraphs turned into prop docs, and specs: nothing opens before `delay`, the close waits `close_delay`, and re-entering the content cancels the close. Check the existing `hover-card.spec.ts` and `tooltip.spec.ts` expectations, which assume instant open. **LANDED 2026-10-15:** `delay` and `close_delay` on the primitive roots, taken from the current Base UI master: HoverCard 600 / 300, Tooltip 600 / 0. The styled Tooltip defaults to 0 / 0 (shadcn's `TooltipProvider` has delay 0 in base, radix and new-york-v4 `tooltip.tsx`); the styled HoverCard keeps Base UI's numbers (shadcn sets none). Timers are Rust-side (`menu_sub::DelayedAction`, cleared on drop, no `setTimeout` through eval); focus, blur and Escape bypass the delays, scroll still closes at once. 13 new tests, 176 related passed.
