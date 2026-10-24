<!-- Investigation report, 2026-10-17, read-only agent run against the 83758d5 debug SSG and the shadcn clone at 30a58d4; scratch paths under /home/user/tgt are session-local. -->

# Overview card link lands at the component page's bottom (read-only investigation)

Deployed debug SSG (83758d5) at :8130; checkout HEAD is f52c4e5 (same code paths). Scripts: /home/user/tgt/scratch-overview/{repro,bisect,back,direct}.js

## What the "overview" is
`Home` (`/`) -> `ComponentGallery` -> card in preview/src/main.rs:3490-3505: `h3 > Link { to: Route::component(name), class: "dx-component-card-title-link" }`.
(The masonry "Showcase" blocks above it have no header links.) The link is a plain router push; no hash, no focus, no autofocus.

## Reproduction (Chromium, desktop 1280x800 and iPhone-13 touch/mobile)
Scroll to a card far down `/`, click its title link, sample scrollY every ~40 ms:
- toast, from Y=29011 (home h=31549): new page h=2456 -> first frame after the swap is scrollY=1656 (= page max, the BOTTOM: "Variants | types"), then it glides to 0 over ~700 ms (zero at ~1.0 s).
- date_picker, from Y=12027: first frame Y=6552 (new h=7352, bottom shows the last demos, "natural language"), glides to 0, zero at ~1.5 s (debug build).
- tabs, carousel: same shape (Y clamps to new max, then glides). Mobile: same (toast 37292 -> 2199 -> 0; tabs 35650 -> 8933 -> 0).
- accordion (home Y=2377 < new max 4166): no bottom frame, just a smooth glide up from 2377.
So: it ALWAYS ends at the top (scrollY=0 after ~1-1.5 s), never stays at the bottom, but a card far enough down shows the BOTTOM of the
component page first, then scrolls up visibly. Rule: flash happens iff old scrollY > new page's max scroll. On a faster (release) build
the swap is quicker, so the bottom frame is the first thing seen. Which demo is at the bottom depends on the page/viewport (I saw "natural
language" at 1280 on date_picker; I could not literally reproduce "date range" -- it is a date_picker variant just above the last ones,
so on a different viewport width/zoom it can be the bottom-most one in view).
Direct load of /component/date_picker/ and /toast/ (no client nav): scrollY=0, no problem. Active element stays BODY (nothing focuses).
Back button from the component page: same defect mirrored -- home renders at 0, then glides smoothly to the saved Y (29016) over ~0.5 s.

## Culprit (stack captured by patching window.scrollTo / scrollIntoView / focus / scrollBy)
Only scroll calls during the nav: `window.scrollTo([0,0])` called from wasm (`__wbg_scrollTo` in preview.js) = dioxus-web 0.7.9
`WebHistory::handle_nav` (src/history.rs:77-81, `if do_scroll_restoration { window.scroll_to_with_x_and_y(0.0, 0.0) }`), called on push.
No focus(), no scrollIntoView, no hash. (carousel `scrollBy` calls are horizontal, inside carousels.)
Back/forward restore is `ScrollPosition::scroll_to` (history.rs:361-366), same plain `window.scrollTo(x, y)` in a rAF.

The amplifier is ambient CSS: **preview/assets/main.css:90-95**
`@media (prefers-reduced-motion: no-preference) { html { scroll-behavior: smooth } }` -- its comment says it exists for in-page `#` anchors,
but it makes EVERY programmatic window scroll animated. Sequence on a push: scrollTo(0,0) starts a smooth scroll from the long home
page -> the router swaps in the short component page -> the browser clamps scrollY to the new max (bottom) -> the animation continues
from there to 0.
Bisect (all three Y traces are in bisect.out):
- inject `html{scroll-behavior:auto !important}`: first frame after swap is already Y=0, zero flash (desktop + mobile, date_picker/toast/tabs).
- `prefers-reduced-motion: reduce`: same, Y=0 at the swap. So the motion-averse never see it; everyone else does.

## It is a class, not one bug (CLAUDE.md: two+ occurrences)
Ambient smooth + programmatic scroll already bit twice more, each patched per instance:
1. preview/src/charts_gallery.rs:307-328 passes `behavior: 'instant'` per call "to override the page's scroll-behavior: smooth".
2. primitives/src/scroll_lock.rs:731 `window.scrollTo(x, y)` on unlock (scroll restore) has no behavior -> animates under the same CSS.
3. This bug, plus back/forward restore (above). preview/index.html's own anchor code already passes `behavior:'instant'` on every aim.

## Proposed construction (ONE rule, subsumes 1-3 and any future window.scrollTo)
Smooth scrolling is opt-in per interaction, never ambient:
- main.css:90-95 -> `html[data-anchor-scroll] { scroll-behavior: smooth }` inside the same reduced-motion media query. No ambient rule.
- preview/index.html click handler (line ~77-90, the capture `a[href^="#"]` listener, which already calls `settle(t)`): set
  `data-anchor-scroll` on <html> there and clear it in `release()` (same lifetime as data-cv-settle). Anchor jumps stay smooth;
  router scrollTo(0,0), history restore, scroll_lock restore, charts tab restore are all instant by default (the charts per-call
  'instant' and its comment can then be deleted).
- Gate `scripts/check-scroll-behavior.sh` (add to scripts/run-gates.sh + the CLAUDE.md gate list): `scroll-behavior: smooth` may appear in
  preview/assets, preview/src, primitives/src CSS only under `[data-anchor-scroll]` or on a non-root element, reasoned allowlist,
  stale entries fail (same shape as check-hover-media.sh).
Not needed: a router scroll policy of our own -- dioxus-web already does top-on-push and restore-on-back; the only defect is the animation.
Mount-focus stealing is NOT involved here (nothing focuses on mount in this nav), so no focus rule is warranted by this bug; the existing
close-on-scroll/preventScroll conventions are unaffected. Hash links: index.html already handles load/hashchange; a router push to a
hash route would still get scrollTo(0,0) first (unchanged, out of scope).

## Row 225 (email-client per-view scroll memory)
Different mechanism (a container scroller inside the demo, keyed by view; restore must be instant). The data-attr rule above helps it
(a programmatic scrollTop set is only affected by ambient CSS if the container inherits it -- it does not, html-only), but the
policy helper is not shared: the router already owns window save/restore. If wanted, a tiny `ScrollMemory { save(key, el), restore(key, el) }`
(HashMap<key, f32>, default top) in preview/src/dashboard/views/email_client/ serves 225; not worth a primitive until a second user.

## Files the fix touches
preview/assets/main.css (rule), preview/index.html (set/clear attr), scripts/check-scroll-behavior.sh (new) + scripts/run-gates.sh + CLAUDE.md,
optionally preview/src/charts_gallery.rs and primitives/src/scroll_lock.rs (drop now-redundant workarounds; no behavior change).
## Proving spec: playwright/nav-scroll-instant.spec.ts (functional; fine on debug builds)
1. `/`, scroll to the last gallery card (y > 20000), click its title link; poll `scrollY` per rAF until the new route mounts: assert
   every sample after the pathname changes is 0 (never >0) -- fails today (first sample = new page max). Run at 1280 and an iPhone viewport.
2. Same with `page.goBack()`: assert scrollY never passes through a value < saved-Y-after-restore ... i.e. the first post-swap sample is the saved Y or 0 -> saved Y in
   one step (no intermediate values).
3. Anchor click on /docs Start sub-item still animates: >=2 distinct intermediate scrollY samples (guards that smooth was not lost).
