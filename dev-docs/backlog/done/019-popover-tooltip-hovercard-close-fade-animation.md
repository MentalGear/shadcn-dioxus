---
id: 19
title: "Popover/Tooltip/HoverCard close-fade animation cut short on the web (popover) arm"
status: "LANDED 2026-09-18. Construction: swapped use_popover_sync for the crate's existing…"
size: "S"
area: theme
updated: 2026-09-18
---

# Row 19: Popover/Tooltip/HoverCard close-fade animation cut short on the web (popover) arm

## Item

Popover/Tooltip/HoverCard close-fade animation cut short on the web (popover) arm

## Status / trigger

**LANDED 2026-09-18.** Construction: swapped `use_popover_sync` for the crate's existing `use_popover_shown_while_mounted` at the three call sites that still used it (`TooltipContentRendered`, `HoverCardContentRendered`, `PopoverNonModalContent`) — the hook already used by `SelectList`/`ComboboxList` (PR #23) to stay mounted through their own exit animation; it never calls `hidePopover()` from the closing path at all, only `use_animated_open` dropping `render` to `false` (and Dioxus actually removing the element) takes something out of the top layer, by which point the CSS close animation has already finished playing. Browser→signal sync (native `toggle` on light-dismiss/Escape) is unchanged, so Rules 2/3 of `top-layer.spec.ts` — whose native-close path bypasses the exit animation on its own, independent of either hook — see the same observable outcome as before. `top_layer.rs` itself untouched: `use_popover_shown_while_mounted` already generalizes across `PopoverKind::Auto` (Popover) and `PopoverKind::Manual` (Tooltip/HoverCard), so this landed as a call-site-only fix (commit `6b57bb3`). Regression test: a new shared sampler `playwright/assert-fade-out.ts` (modeled on `assert-close-animation.ts` + `accordion-animation.spec.ts`'s rAF sampling loop) plus one test per component (`tooltip.spec.ts`/`hover-card.spec.ts`/`popover.spec.ts`) samples the closing element's `opacity`/`display` every frame and asserts it stays displayed with decreasing opacity for a few frames, reaches ~0 before unmount, and unmounts within ~1.5s — demonstrated RED on pre-fix `tooltip.rs`, GREEN restored, during integration round 1. **Surprise found integrating:** `popover.spec.ts`'s non-modal case drives the `top_layer` oracle fixture's raw `#stack-popover-*` instance, which deliberately carries no themed CSS at all — there is no close animation to observe there regardless of this fix (`use_animated_open` correctly unmounts immediately with nothing to hold), so that one sub-case cannot pass against its current test subject. Retargeting it at a themed, non-modal Popover demo variant is now **done**: Lane F added a themed `non_modal` Popover demo variant (`preview/src/components/popover/variants/non_modal/mod.rs`, `is_modal: false`, an "Open popover" trigger; commit `c7060c8`, cherry-picked from `30ffc8a`) and retargeted `popover.spec.ts`'s close-fade regression at it (URL `/component/?name=popover&variant=non_modal&`, trigger by label "Open popover", content id via `getByRole("dialog")`, close via `el.click()` evaluate so native light-dismiss does not pre-empt the animation; commit `c7289de`, cherry-picked from `2e56c85`) — confirmed passing live in integration round 2's targeted Playwright run ("popover close-fade non-modal arm green").

## Size

S

## Detail

Row 7 (Tooltip's fade-out CSS, cherry-picked from Wervice's upstream PR #293) is sequenced after this row in the same lane, since the CSS has nothing to hold open without this construction; HoverCard and Popover's own symmetric close-fade CSS is accounted on row 7
