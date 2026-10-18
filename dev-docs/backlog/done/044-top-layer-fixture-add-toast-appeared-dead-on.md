---
id: 44
title: "top_layer fixture: \"Add toast\" appeared dead on iOS 18 (device report 2026-09-03). Root-caused last round…"
status: "Landed 2026-09-04"
size: "S"
area: tests
updated: 2026-09-04
---

# Row 44: top_layer fixture: "Add toast" appeared dead on iOS 18 (device report 2026-09-03). Root-caused last round…

## Item

**LANDED 2026-09-04.** `top_layer` fixture: "Add toast" appeared dead on iOS 18 (device report 2026-09-03). Root-caused last round (not reproducible in headless Chromium, desktop or iPhone-emulated `.tap()`: the toast was created, `permanent`, on-screen, no console errors — device-only report), fixed this round, two independent construction pieces. **(a)** `primitives/src/toast.rs` now injects a minimal always-applied fallback stylesheet (`ensure_toast_base_styles`, mirroring `top_layer.rs`'s `ensure_anchor_positioning_styles`), all rules `:where()`-wrapped at zero specificity, so raw/unstyled `ToastRegionRendered`/`Toast` markup gets a deterministic cross-engine box instead of depending on the engine's popover UA defaults; verified line-by-line that the themed `.dx-toast-container[popover]`/`.dx-toast` rules already declare every property it sets, so themed pages are unchanged. This was chosen over the row's other originally-listed construction option (routing the fixture through the themed `toast::ToastProvider` wrapper) because `scripts/check-preview-composition.sh` exempts `top_layer/component.rs` either way and that fixture's raw composition is deliberate. **(b)** Both `ToastRegionRendered` arms now fold a caller `style` with `--toast-count` via `crate::fold_style_attributes` (the `top_layer::anchored_content_attributes` shape) instead of letting `merge_attributes`' last-wins `style` rule drop the counter. The identical clobber on `Toast`'s own `--toast-index` was fixed at the same time — no caller passes a per-toast `style` today, so that half is prevention rather than an observed bug. New `toast.spec.ts` oracle asserts `boundingBox()` non-null/non-zero/in-viewport on the themed demo and the fixture, plus a `--toast-count` computed-style check; **all pass in Chromium.** Honest note: Chromium never reproduced the original iOS symptom, so these bounding-box checks are forward-looking cross-engine coverage, not a demonstrated red-to-green on the browser that actually showed the bug

## Status / trigger

Landed 2026-09-04

## Size

S

## Detail

Original root-cause detail, carried from the previous round: the fixture's toast had whatever box the engine's default flow gave an unstyled `div[role=alertdialog]` inside `ol > li` inside a `popover=manual` region — Chromium happened to give it ~105×40 px on-screen; WebKit's default for the same unstyled, top-layer-promoted markup was the unverified suspect at the time. This session's environment gained working Firefox and WebKit installs for the first time (see the Fresh-session order environment note above), but the in-progress firefox+webkit calibration run's result is unknown as of this writing (row 4) — so this fix's cross-engine claim still rests on construction plus the Chromium-only oracle pass above, not an executed WebKit confirmation
