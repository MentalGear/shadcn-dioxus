---
id: 491
title: "Drawer: add shadcn's scrollable content demo"
status: "DONE 2026-11-06, landed in PR #165. scrollable_content now ends its body in a \"Feedback\" Input, and…"
size: "S"
area: components
updated: 2026-11-06
---

# Row 491: Drawer: add shadcn's scrollable content demo

## Item

Add shadcn's Drawer demo where the content is long and scrolls inside the drawer while the header and footer stay in
place. The demo must keep working with the iPhone keyboard inset of row 480: a field in the scrolling body, with the
keyboard up, is brought into the visible band.

## Status / trigger

DONE 2026-11-06, landed in PR #165. `scrollable_content` now ends its body in a "Feedback" `Input`, and
`playwright/drawer-scrollable-content.spec.ts` (phone width, light and dark, axe) asserts: the body is the scroll
container (`overflow-y: auto`, panel not scrolling), header and footer keep their position while the body scrolls,
`window.scrollY` stays put (gap 1); and with a stubbed `visualViewport` keyboard of 336px (`data-dx-keyboard`,
`--dx-keyboard-inset`) the focused field, reached with `preventScroll`, lies inside the visible band and the body, the body
scrolled and the footer ends above the keyboard (gap 2). It passed on the first run: `keyboard_inset.rs` already handles the
field, so the spec stays as a regression guard; no primitive change. `notes.md` is rendered as the demo's description like
the other variants' (gap 3). The upstream name stays unconfirmed (see Detail). Row 480's device check stays the final proof.
Green on chromium only: the box's pinned Playwright 1.60 wants chromium 1223 while 1243 is installed, so the lane ran with a
`PLAYWRIGHT_BROWSERS_PATH` of symlinks.

## Size

S

## Detail

**Finding at HEAD (2026-11-01): the demo already exists, so the job is smaller than the row reads.**

- **The demo is built.** `preview/src/components/drawer/variants/scrollable_content/` came in under row 188 (batch
  demos-a, 2026-10-19). It is listed in `variants/order.txt` and covered by
  `playwright/demos-menu-overlay-floating.spec.ts` (the case "drawer/scrollable_content: the body scrolls, the footer
  stays visible", light and dark).
- **The name is not confirmed upstream (name to confirm).** On 2026-11-01 the shadcn drawer pages
  (`https://ui.shadcn.com/docs/components/base/drawer` and `https://ui.shadcn.com/docs/components/drawer`) have no
  example titled "Scrollable Content". Their examples are: Open Drawer, Open Left Drawer, Swipe Handle, Nested, Non
  Modal, Open Snap Drawer, and Edit Profile (Responsive). The closest upstream pattern is the "Custom Sizes" code
  snippet: a middle region with `flex-1 overflow-y-auto` between a fixed header and footer, which is what the variant
  implements. The fetch was summarised by a small model, so the lane re-reads the page before it relabels the demo.

So this row is not a new demo. It closes the gaps between the existing demo and what the row asks for.

**What the demo does at HEAD.** A right-side drawer (`DrawerSide::Right`) with a header, a body and a footer. The body
is a `role="region"` (`aria-label` "Release notes list", `data-testid` `drawer-scroll-body`) with
`flex: 1 1 0%; min-height: 0; overflow-y: auto`, holding 30 rows. The header and the `DrawerClose` footer stay fixed.
The variant's `notes.md` is the one-sentence docs entry.

**Gaps against the row.**

1. **The page does not scroll.** The spec asserts that the body scrolls and the footer is in the viewport. It does not
   assert that the page behind the drawer stays put (`window.scrollY` stays 0 while the wheel or a scroll is aimed at
   the body, the page scroll lock holds). Add that assertion.
2. **No keyboard case (row 480).** The body holds no field, so the iPhone keyboard inset is never exercised. Put an
   `Input` at the end of the body (or a second variant whose body ends in a form), and assert that with the keyboard
   inset emulated (`data-dx-keyboard` and `--dx-keyboard-inset` on the panel, as `primitives/src/keyboard_inset.rs`
   writes them) the focused field lies inside the visible band `[top, 100% - inset]`. Row 480's device check stays the
   final proof; the spec is the emulated one. If the emulation cannot drive `keyboard_inset.rs`, say so in the lane
   report rather than weakening the assertion.
3. **Docs entry.** Confirm `variants/scrollable_content/notes.md` is what the Drawer page renders as this demo's
   description, the same as the other variants' `notes.md`.

**Parity conventions (rows 188 and 232).**

- A demo is its own `variants/<name>/` directory; `main` is not touched (so no spec text moves).
- The directory is listed in `variants/order.txt`.
- The `mod.rs` doc comment names the shadcn demo it follows and says what the demo shows, not how it was added.
- The spec opens the demo in its own frame, runs in light and dark, runs axe, and finds controls by role inside the
  frame, never by demo copy elsewhere on the page.
- No installation prose goes into the demo or its docs (row 232: installation lives in one place).

**Files the lane owns.** `preview/src/components/drawer/variants/scrollable_content/` (`mod.rs`, `notes.md`), the
`variants/order.txt` line if the demo is renamed, and `playwright/demos-menu-overlay-floating.spec.ts` (or a new
drawer spec beside it). It does not touch `primitives/src/keyboard_inset.rs`; a fix there belongs to row 480.

**Proof.**

- Body scrolls, footer stays in the viewport, and `window.scrollY` stays 0 (the page does not scroll), light and dark,
  with axe.
- The keyboard case: the focused field sits in the visible band with the emulated inset. Red first if the field is
  covered; if it already passes, the spec stays as a regression guard and the row says so.
- Gate: `scripts/run-gates.sh --fast` in the lane; the full run belongs to the merge batch.
