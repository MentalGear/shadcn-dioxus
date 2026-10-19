---
id: 99
title: "anchor_name_style set as a bare literal beside a spread — a duplicate-style class across the anchored-trigger…"
status: "Found 2026-09-21 by Rule 4c on its first run (NavigationMenuTrigger), then widened to the whole…"
size: "DONE — all 9 sites now route through…"
area: overlays
updated: 2026-09-21
---

# Row 99: anchor_name_style set as a bare literal beside a spread — a duplicate-style class across the anchored-trigger…

## Item

`anchor_name_style` set as a bare literal beside a spread — a duplicate-`style` class across the anchored-trigger family, and a second, quieter bug at the sites that looked safe

## Status / trigger

Found 2026-09-21 by Rule 4c on its first run (`NavigationMenuTrigger`), then widened to the whole family before fixing

## Size

DONE — all 9 sites now route through one construction

## Detail

**Two distinct defects, and the second is the one that would have been missed.** (a) Four sites set `style: anchor_name_style(...)` as a bare literal on an element that also spread caller attributes — `hover_card.rs`, `menubar.rs`, `navbar.rs`, `navigation_menu.rs` — so a caller supplying `style` produced two `style` attributes. `navbar.rs`'s own comment claimed "Merged (caller-wins, deduped)", which is true for its `id` and false for its `style`; the comment is why this looked like a single-site bug. (b) The five sites that were safe from the duplicate (`context_menu.rs` sub, `dropdown_menu.rs` ×2, `popover.rs`, `tooltip.rs`) were silently **losing the anchor binding** whenever a caller supplied `style`, because `merge_attributes` folds only `class` (`join_class`) and replaces every other name outright, later-list-wins. That failure is invisible: no duplicate attribute is emitted, so no oracle can see it. **This is also why the obvious fix was wrong.** The initial brief proposed moving the literal into the merged base list, mirroring `TooltipTrigger` — which would have fixed (a) at the four sites while giving them (b), a worse trade, since (b) fails every time a caller sets `style` and leaves nothing to detect. The lane established the replace-not-fold behaviour with a new permanent test (`style_attribute_is_overwritten_not_folded`) before proceeding, and built `top_layer::anchored_trigger_attributes` instead — the trigger-side mirror of the existing, proven `anchored_content_attributes`, using `fold_style_attributes` so the anchor name is always kept and the caller's declarations still apply after it. Applied at all nine sites, including retrofitting the five that were only affected by (b). Evidence: static analyzer agreed with a full source read on which four were acutely defective; 7 new synthesized `style` cases in the Rule 4c oracle; full hydration-parity run 40 passed / 1 failed (the deliberate `selftest:unfixed:id`); an independent quote-aware re-tokenisation found 0 duplicate-attribute tags across 93,175 start tags over the home page and all 65 component pages; row 93's guard baseline shrank by exactly 4.
