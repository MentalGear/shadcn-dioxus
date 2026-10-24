---
id: 407
title: "Load flakes (docs sidebar shift, avatar demo re-centring, non-modal popover scroll, carousel edge band) and spec runs that serve unknown URLs like the site"
status: "Done on lane/flake-roots. Trigger for reopening: one of these specs red under load on an unchanged…"
size: "M"
area: tests
updated: 2026-10-24
---

# Row 407: Load flakes (docs sidebar shift, avatar demo re-centring, non-modal popover scroll, carousel edge band) and spec runs that serve unknown URLs like the site

## Item

Three specs went red under load and green alone: `layout-stability` on `/docs/` (the desktop sidebar moved after the
first paint), `popover-non-modal.spec.ts` :187 and :216 (wheel and keyboard scrolling with the popover open), and
`carousel.spec.ts` :1583 (`releasedBy` "slow-silence" where "smooth-decay" was expected). Separately, lanes ran
`pnpm run test:changed` against a build served with `python3 -m http.server`, whose bare 404 made
`theme-picker-phone`'s not-found routes time out.

## Status / trigger

Done on `lane/flake-roots`. Trigger for reopening: one of these specs red under load on an unchanged component.

## Size

M

## Detail

Each was reproduced on a debug `lane-ssg` build before it was fixed, and the fix verified under the same load.

**Not-found routes.** `playwright/tools/site-server.sh` is the one precondition `test:changed` and `run-suite.sh`
call: with `SSG_SITE_DIR` and no `PLAYWRIGHT_BASE_URL` it serves the dir itself (`emit-404.sh` + `serve-spa.py` on a
free port, stopped on exit); otherwise it requests `/__dx-unknown-route__/` and refuses (exit 2) unless the answer is
status 404 with the app shell. Checked against `http.server` (refused), `serve-spa.py` (accepted) and a dead port
(refused).

**Carousel edge band (`carousel.spec.ts`).** `playWheel` spaced its synthetic events with `setTimeout(16)`; on a
loaded box one late timer is a gap over `GAP_MS` (100 ms), which the band rightly reads as a new gesture, and the
tail split off that way (18, 16, 15, 14, 13) has a peak under `SLOW_PEAK_PX` and fewer than `RUNS` decaying pairs, so
its record ends in "slow-silence". Reproduced exactly by a 150 ms main-thread stall 400 ms into the stream on the old
spec; the fixed spec passes with the same stall. Construction: a stream is dispatched in one task (no gap can open
inside it; every rule of the band reads time only as silence, so "no silence inside, `tailMs` after" is what each test
means); `dtMs` is gone. Not subsumed: `wheel-replay.ts` replays recorded inter-event times on purpose.

**Non-modal popover scroll (`popover-non-modal.spec.ts`).** Red 4 of 12 on a debug build with six busy loops and two
workers. :187 wheeled 800 px with the trigger at y = 475 of a 720 px viewport, so the anchor left the viewport and the
popover closed, which is its contract (`ScrollClose::AnchorLeavesViewport` after the 300 ms quiet window); the check
passed only when it ran before that close. :216 pressed End while PageDown's smooth scroll was still in flight (an
in-page frame log: End dropped, scrollY 577 of 3015). `scrollAndSettle` waits for a scroll to move and then hold for 10
frames; the wheel tests scroll only as far as keeps the trigger in view (`scrollKeepingAnchor`). After: 66/66 under the
same load.

**Docs sidebar (`layout-stability.spec.ts`, `/docs/`).** Reproduced with `LS_CPU=4` and `6`: 265 sidebar boxes moved
from y = 53.2 to 60 after the first paint. The navbar's content is 53.2 px tall, the sidebar is `position: sticky; top:
var(--dx-navbar-height)` (60 px), and the shell's min-height is exactly the sidebar's height, so `sticky` can only push
the sidebar down the 6.8 px once the main column has been parsed far enough to make the shell taller. Construction: the
navbar's `min-height` is the token, so the sidebar's natural offset is its sticky `top` and nothing waits on the
shell's height (the navbar is 6.8 px taller than before; the settled sidebar position is unchanged). Same class, found
once that was gone: the nav-sort pill was a flex row that gave the A-Z / Category toggles the whole width until the
chevron trigger after them was parsed, and the toggles did the same for their two items; both are now grids with
fixed columns. The spec's progressive-parse clause also counted text still arriving in an open element (a paragraph,
an inline `code`, an empty `strong` on its baseline) as a move; an element that was open in the parser on the previous
frame and only grew, from its left or right edge, is now filling, not moving.

Not subsumed (found at `LS_CPU=4` on `/docs/`, 2 of 2 runs; `LS_CPU=6` was green 2 of 2, and the spec's default
`LS_CPU=1` is what the suite runs): the navbar's right-hand links move 32-40 px left at 760-820 px when the theme
picker's trigger after them is parsed (the same "later sibling takes space" shape as the nav-sort pill, in the
navbar's flex row), and an inline `code` whose padding box is painted before its text moves 4 px right as it fills.
Both need the same fixed-from-the-first-byte construction for the navbar row; left for a follow-up row.

**Avatar page (`layout-stability.spec.ts`, `/component/avatar/`, the batch-16 release red at 448 px dark).** Reproduced
every run with `LS_STEP=32 LS_CPU=4` (light and dark, 30+ of 64 widths): the page is painted while the HTML is still
arriving, and the demo row (`justify-content: center`) is drawn with one or two avatars and re-centres as the rest are
parsed (12-116 px). The RTL toggle above it did the same as its label arrived. Not batch 16's (the demo is unchanged
against main): the progressive-parse class above, which load exposes. Construction: a component page's first paint
waits until the main demo frame has been parsed (`document::Link { rel: "expect", href: "#component-preview-frame",
blocking: "render" }` in the SSR head, Chromium 124+, ignored elsewhere), so nothing above or in the first demo can be
re-laid out by HTML that has not arrived yet. After: the avatar (light, dark) and button sweeps green at `LS_STEP=32
LS_CPU=4`. Cost, median FCP of 5 loads at 1280 px: avatar 128 -> 132 ms and button 104 -> 104 ms unthrottled,
avatar 360 -> 420 ms and button 252 -> 320 ms at 4x CPU. Not gated (no failure seen there at the suite's settings):
the home page (its masonry has its own construction, row 214), `/docs/` and `/demos/`; the navbar-row member above is
gated on component pages only.

**Every page with a top group, and a sweep that does not need load (owner decision 2026-10-24: hold the first paint
until the top group is parsed).** `FirstPaintGate` (`preview/src/main.rs`) is the one construction: an empty marker
placed where a page's top group ends plus the render-blocking `expect` link for it, on component pages (after the main
demo), home (after the masonry), `/docs/` (end of its fourth section) and `/demos/` (after the card grid). The
`layout-stability` sweep loads every width through `slowDocServer` (the first 250 KB of the body in 8 KB pieces 20 ms
apart), so the class shows on every run: the batch-16 build fails all five `/docs/`, `/demos/` and component groups on
an idle box (sidebar 53.2 -> 60 at every desktop width, the avatar row re-centring), and this tree passes all seven
groups (6.4 min at 2 workers).
