---
id: 144
title: "Opt-in hidden=\"until-found\" for Collapsible, Accordion and Tabs"
status: "LANDED 2026-10-05 (round 3; owner-approved) -- from the virtualization survey (sections 1.9 and 6)"
size: "M"
area: inputs
updated: 2026-10-05
---

# Row 144: Opt-in hidden="until-found" for Collapsible, Accordion and Tabs

## Item

Opt-in `hidden="until-found"` for Collapsible, Accordion and Tabs

## Status / trigger

**LANDED 2026-10-05 (round 3; owner-approved)** -- from the virtualization survey (sections 1.9 and 6)

## Size

M

## Detail

Closed content is unmounted in all three (Collapsible unless `keep_mounted`, Accordion after the close animation, Tabs renders an empty `div[hidden]`), so Ctrl+F cannot find it, a `#fragment` cannot open it, and SSR/SSG HTML omits it. `hidden="until-found"` is the one native member of the content-visibility family that adds a user-visible capability rather than speed (probe: fragment navigation fires `beforematch`; `window.find` finds `until-found` text and not `display: none`). **Sketch, default unchanged:** Collapsible keeps children mounted under `hidden="until-found"` (its `keep_mounted` today sets nothing and the styled layer would show them -- a latent bug with no spec); Accordion sets it after the close animation instead of unmounting; Tabs renders inactive panels and must delete the author `display: none` at `tabs/style.css:118`, which defeats the UA rule; a `beforematch` listener (no typed `onbeforematch` in Dioxus 0.7: a small `document::eval` bridge or `onmounted`) opens the panel. Cost: every panel mounted and hydrated, which is why it is opt-in. Support: Chrome 102, Firefox 139 (events; 148 full), Safari 26.2 partial (reveals without scrolling, WebKit 304174); older engines treat it as plain `hidden`, so there is no regression path. Headless `window.find` did not fire `beforematch`: verify the Ctrl+F path by hand. **2026-10-05 addendum -- Owner-approved 2026-10-05; in progress in this session.** The windowed-list analogue (searchable text placeholders carrying `hidden="until-found"`) is row 153. **Landed 2026-10-05 (round 3):** a `hidden_until_found` prop (default false) on Collapsible, Accordion and Tabs, sharing `hidden_attribute`, `use_beforematch` and `PanelHidden` from `primitives/src/collapsible.rs`; `until_found` demos and a docs section with the browser-support table. Collapsible keeps closed content mounted as `hidden="until-found"`; Accordion sets it after the close animation instead of unmounting; Tabs mounts inactive panels the same way, and `tabs/style.css` no longer applies its author `display: none` to `hidden="until-found"` panels (it would defeat the UA rule) and takes those panels out of flow at zero size so the themed card padding does not show. **Side finding, fixed:** Collapsible `keep_mounted` closed content used to be shown (the latent bug this row named); it now carries `hidden=""`, and `hidden_until_found` overrides `keep_mounted`. **Specs:** `collapsible.spec.ts`, `accordion.spec.ts` and `tabs.spec.ts` drive the reveal by a real `#fragment` navigation and a synthetic `beforematch`, because headless Chromium's `window.find` finds until-found text but does not fire `beforematch`. **Still unverified, by hand only:** the real Ctrl+F path in a headed browser (not done in round 3).
