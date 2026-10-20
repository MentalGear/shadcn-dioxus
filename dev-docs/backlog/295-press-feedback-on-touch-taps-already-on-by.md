---
id: 295
title: "Press feedback on touch taps: already on by default, so check why taps don't show it"
status: "PARTLY LANDED 2026-10-20 (theme-touch lane 1ba0b2e5, cf64037a); device re-test open. Row 205…"
size: "S (or none)"
area: theme
updated: 2026-10-20
---

# Row 295: Press feedback on touch taps: already on by default, so check why taps don't show it

## Item

"Press feedback default on, on touch taps."

## Status / trigger

**PARTLY LANDED 2026-10-20 (theme-touch lane 1ba0b2e5, cf64037a); device re-test open.** Row 205 (LANDED 2026-10-14) **already ships press feedback on by default**, for every pointer. Turning it off takes `html[data-dx-press-feedback="off"]`, set by the theme picker switch (`theme.rs:~625`).

## Size

S (or none)

## Detail

- **Today:** the rule is `dx-components-theme.css:966-1030`: `:active { scale: 0.98 }` plus an 8% wash. It has zero specificity, so a component with its own `:active`, `background` shorthand or `transition` silently overrides it (the comment there says so).
- `index.html:63` is NOT a standing listener: the `touchstart` there belongs to `aim()` and is removed after the 1 s scroll aim, so the page has no page-lifetime `touchstart` (see "Open").
- **Proposed row:** check the components where a tap shows no feedback on a phone, and fix those overrides.
- Touch-only alternative (not the default): a one-line gate on `data-dx-modality="touch"`, and it reverses row 205's design.
- Owner default (2026-10-19): keep it on for every pointer and fix the components that miss it.
- If a particular control showed nothing when you tapped it, tell me which one and I'll name it in the row.

## Findings (theme-touch lane, 2026-10-20; debug SSG of main f62953ad, Chromium touch emulation at 390px)

Audit: every distinct `dx-` control signature on 85 routes, `:active` forced through CDP, computed `scale` / `background-image` / `transition` read (76 signatures). Two causes, both by construction in the theme:

1. **The wash never painted on 23 control families.** The wash is a zero-specificity `background-image`; any component that writes the `background` shorthand at rest (Tabs trigger, Select trigger, Menubar / Navbar / Navigation Menu triggers and links, Calendar cells, Carousel buttons and dots, Toolbar button, Marker, Attachment, Trace, Combobox chip/clear, ...) resets it at a higher specificity. Those showed only `scale: 0.98`, under 1px on a 24px trigger. Fix: the wash rule uses the same selectors without `:where()` (specificity about (0,4,2), above any single component rule; none of the audited controls has a rest `background-image` of its own, and no component declares its own `:active` `scale` or `background-image`); silence or retune it with `--dx-press-fill`.
2. **A tap is shorter than the ramp.** Press-in was a symmetric 100ms `scale` transition, and 77 component sheets have their own `transition` that beats the theme's zero-specificity one (Button: `all 150ms`). Measured with a held emulated touch: 30ms -> scale 0.9993, 60ms -> 0.9965 (nothing visible), 100ms -> 0.983. Fix: the press lands instantly (`--dx-press-in-duration: 0s`, `!important` on the `:active` rule's `transition-duration` because a specificity fight loses to every component) and releases over the component's own transition, else `--dx-press-duration` (now 200ms). Same measure after: 30ms hold -> full 0.98 for the touch and a 9-frame tail on Button; Tabs/Menubar/Toolbar wash frames 0 -> all.

Spec: `press-feedback.spec.ts` (instant press-in under a component transition, a 60ms press leaves a tail, the wash paints on tabs, menubar, toolbar, navigation-menu, carousel-next; the existing switch-off and reduced-motion tests keep their meaning). Note for spec authors: `client.detach()` right after the touch-end CDP event drops the event and leaves `:active` set on about 1 release in 3 (also on the old code), so the helper no longer detaches.

## Open

- **iOS `:active` without a touchstart listener (cannot be reproduced here).** Chromium emulation proves the CSS (what is applied while `:active` matches, and for how long); it cannot prove WebKit raises `:active` on a tap at all. Historically iOS Safari needed a `touchstart` listener on the element or an ancestor for non-link, non-button elements (`div[role=menuitem]`, options, tabs), and delays `:active` ~100ms to tell a tap from a scroll. The page has no standing listener. Recommended follow-up (files outside this lane: `primitives/src/modality.rs`, `PREPAINT_JS` in `preview/src/theme.rs`, `preview/index.html`, which must stay byte-identical): add one passive page-lifetime capture `touchstart` no-op to the modality writer, with its `listener-ok:` marker. Do it only if the device re-test below fails.
- **Device re-test for the owner (real iPhone, Safari):** on `/component/tabs/`, `/component/menubar/`, `/component/toolbar/`, a Button and a dropdown-menu item: tap quickly (about 50ms) and hold for a second; the control should dip and show a darker wash for the touch, and ease back for 150-200ms. Report any control that shows nothing, and whether quick taps show anything.
- **29 control families still snap back on release** (their `transition` list names neither `all` nor `scale`: switch, toggle, toggle-group item, color-picker button, slider thumb, breadcrumb link, dropdown trigger, item, marker, menubar / navbar / navigation-menu triggers, tag-group and drag-and-drop remove buttons, hero CTA, footer link, bubble, trace, carousel next/previous/indicator, date-picker trigger, Select trigger, ...). They now show the press for its whole duration but have no tail. A gate (`every dx control sheet's transition list includes scale or all`, shrink-only baseline) is the by-construction answer if the owner wants the tail everywhere; not built here (touches 29 sheets).
- Strength: `--dx-press-scale` stays 0.98 (row 205's value). If the device test finds it too faint, 0.97 and a 10-12% wash are one-token changes.
- Screenshots (held press, 3x): `/home/user/tgt/shots-theme-touch/295-{tabs,menubar,toolbar}-{rest,pressed}-{before,after}.png`.
