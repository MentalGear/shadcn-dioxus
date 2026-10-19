---
id: 26
title: "NOT REPRODUCED — re-investigated 2026-09-04, first session with all three engines available. Original claim…"
status: "Closed as not-reproduced. Workaround removed 2026-09-04. Reopen only with a fresh, engine-named…"
size: "M"
area: tests
updated: 2026-09-04
---

# Row 26: NOT REPRODUCED — re-investigated 2026-09-04, first session with all three engines available. Original claim…

## Item

**NOT REPRODUCED — re-investigated 2026-09-04, first session with all three engines available.** Original claim: after a `Link` navigation the outgoing `ComponentDemo` page's content — including a `popover`-promoted, still-open `DropdownMenuContent` — stayed in the DOM beside the new route's content, so two same-named `role="menuitem"` controls could collide in the accessibility tree. Probed directly (DOM inspection and a `parentElement` walk, not just the a11y tree): deliberately open the DropdownMenu on `/component/?name=dropdown_menu&`, click the real `.dx-navbar-brand` `Link`, then count the outgoing route's markers. **chromium / firefox / webkit all report 0** `.dx-dropdown-menu-content` and 0 demo-container elements afterwards; `document.body.innerHTML.length` is identical between the with-overlay and without-overlay variants (208454/208456 chromium, 208616/208616 firefox, 208721/208732 webkit), so nothing extra is retained when a top-layer popover is open. The only remaining `[role="menu"]` elements belong to the NEW route (the home gallery's own Menubar/Navbar demo cards, all `data-state="closed"`). Not a headless/CSR-dev-server artifact — it does not reproduce in any engine. **The stated precipitating event does not hold either:** the row blamed `dropdown-menu.spec.ts`'s `body.click({x:0,y:0})` landing on the nav-brand link, but `document.elementFromPoint(0,0)` resolves to `<nav class="dx-preview-navbar">`'s own background — the brand link's box starts at `x≈81.5, y≈17` — and replaying that exact sequence produced no navigation at all and exactly 1 match for the unscoped locator. `preview/src/main.rs`'s router/layout wiring is unchanged since the row was filed (2026-09-02), so this is not a since-fixed regression: the original collision looks to have been a misdiagnosed one-off

## Status / trigger

**Closed as not-reproduced. Workaround removed 2026-09-04.** Reopen only with a fresh, engine-named repro that inspects the DOM directly rather than inferring a leak from a Playwright strict-mode collision

## Size

M

## Detail

Found 2026-09-02 as a strict-mode Playwright collision while updating `dropdown-menu.spec.ts` for row 24 (`getByRole('menuitem', {name: 'Edit'})` resolved to both the real DropdownMenu item and a leaked `Menubar` demo trigger after the test's own outside-click step landed on the nav-brand link); worked around in that one spec by scoping the locator to `[role="menu"][data-state="open"]` rather than fixed there — orthogonal to the ARIA-role migration that test was being updated for. **2026-09-04: the workaround itself is now removed**, not merely left in place as harmless — this row's own re-investigation didn't just fail to reproduce the leak, it disproved the workaround comment's *stated mechanism* (the outside-click landing on the nav-brand link, triggering navigation): replaying that exact sequence produced no navigation at all. A locator guard whose own justification is disproved, not just unconfirmed, is no longer "harmless to leave," so `dropdown-menu.spec.ts` now clicks the plain `getByRole('menuitem', {name: 'Edit'})` locator directly — safe as a plain locator too, since `dropdown_menu` has a single demo variant on that page and so cannot be ambiguous
