//! JS anchor-position fallback / self-check (`use_anchor_position_fallback`).

#[cfg_attr(
    not(feature = "web"),
    expect(unused_imports, reason = "only the web build uses it")
)]
use super::*;

/// JS-measured static positioning fallback/self-check for engines where CSS
/// Anchor Positioning doesn't land the content next to its trigger. It is
/// *required*, not merely a nice-to-have: once an element
/// is promoted to the top layer via `popover`, its containing block becomes
/// the viewport regardless of its own `position` value (see
/// [`position_anchor_style`]'s doc) -- which means the `[data-side]`
/// rules in `tooltip`/`hover_card`/`popover`'s `style.css` (written for the
/// DOM-relative `position: absolute`) do not gracefully degrade into
/// "roughly right, just not collision-aware" on an engine that can't resolve
/// `anchor()`; they degrade into detached from the trigger entirely -- the
/// "tooltip not working" symptom this prevents.
///
/// Deliberately does *not* gate on `CSS.supports('anchor-name: --a')` alone,
/// unlike the CSS `@supports` blocks this pairs with (those stay -- they're
/// what makes Chromium's positioning correct from the very first layout,
/// before this effect's synchronous correction below even runs). On Firefox,
/// `CSS.supports('anchor-name: --a')` reports `true` (the property
/// parses), yet a popover-promoted element's `anchor()`-based `top`/`left`
/// can still fail to resolve against its trigger -- a real, engine-specific
/// popover/anchor-positioning integration gap that a syntax-support check
/// cannot see. So instead this *measures the outcome*: it compares the
/// content's actual position to the trigger-relative position this
/// component's own `[data-side]`/`[data-align]` contract promises, and
/// only overrides with inline styles when they disagree by more than a
/// couple of pixels (rounding/subpixel noise). On an engine where `anchor()`
/// already resolves correctly (this repo's Chromium),
/// the computed and actual positions match and nothing is
/// touched -- no extra reflow, no risk of fighting a working anchor
/// resolution.
///
/// ## The flip contract (dev-docs/backlog.md row 10)
///
/// The stylesheets this pairs with (`tooltip`/`hover_card`/`popover`
/// `style.css`, `@supports` blocks) declare
/// `position-try-fallbacks: flip-block, flip-inline` on the anchored
/// `[popover]` rule -- the CSS Anchor Positioning spec's own collision-
/// avoidance primitive
/// (<https://www.w3.org/TR/css-anchor-position-1/#fallback-var>). On an
/// engine that implements it, a preferred placement that doesn't fit the
/// viewport can legitimately paint at the *opposite* side on the relevant
/// axis instead -- e.g. `[data-side="bottom"]` rendering physically above
/// its trigger. That is intentional disagreement with a naive "actual
/// equals `[data-side]`'s own formula" check, and this function runs after
/// that flip has already painted (see the mount-order/timing note further
/// down) -- so this function computes **both** candidate positions (the
/// primary side's own formula, and the same formula with only the relevant
/// axis's side swapped -- top/bottom sides flip on the block axis,
/// left/right sides on the inline axis, matching `flip-block`/`flip-inline`
/// exactly) and accepts the painted position if it matches *either* one.
/// Only when it matches **neither** -- meaning this engine's
/// anchor-positioning integration failed outright, the case this function
/// exists for -- does it override with inline styles at all. This is
/// the one load-bearing rule here: re-checking only the primary formula
/// would see a legitimate CSS flip as "wrong" and stomp it back to the
/// off-viewport primary position, fighting the platform's own fix.
///
/// When it does override (no working anchor-positioning engine at all), it
/// makes the *same* flip decision itself, from plain
/// `getBoundingClientRect()`/`window.inner{Width,Height}` viewport math:
/// if the primary placement would overflow the relevant viewport edge, it
/// uses the flipped placement instead, so engines without CSS Anchor Positioning get the flip too
/// (dev-docs/backlog.md row 10). Resizing WIDTH to fit stays out of scope
/// (a too-wide overlay is rare, and this crate's own themed listboxes/menus
/// already cap their own width independently, e.g. `select/style.css`'s
/// `max-width: calc(100vw - 2rem)`) -- but see "Inline-axis shift" below
/// for the one piece of horizontal shift this function *does* do, and
/// "Vertical shift + size clamp" further below for the same treatment on
/// the other axis, HEIGHT included: unlike width, a too-*tall* overlay is
/// the ordinary case for this crate's own listboxes/menus (more options
/// than a short viewport can hold), so resizing -- capping height, with
/// the content scrolling inside that cap -- is in scope there.
///
/// ## Inline-axis shift
///
/// Flip (both the CSS `position-try-fallbacks` primitive this function
/// mirrors, and its own viewport-math equivalent just above) only swaps
/// `side` to its axis opposite -- block for top/bottom, inline for
/// left/right (the `opposite` map above). For a `side="top"`/`side="bottom"`
/// placement, the horizontal position comes entirely from `align`
/// (start/center/end, computed once against the trigger's own rect in
/// `place()` above); neither CSS (this crate's `@supports` block declares
/// no shift primitive) nor flip checks it against the viewport. A
/// center-aligned overlay wider than the room its trigger has on one side
/// would clip against the viewport's edge on a small screen (iOS Safari has
/// no CSS Anchor Positioning at all, the engine this whole function exists
/// for; the `top-layer.spec.ts` `edge-bottom-*` fixture cases of `Tooltip`
/// and non-modal `Popover` land at `left: -192px`/`-194px` without it) --
/// and `flip-block` could never be the remedy: `side="bottom"` only ever
/// flips to `side="top"`, which changes `top`, never `left`.
///
/// So on this JS-fallback path (below, in the "neither matches" branch),
/// after the flip decision, `target.left` is clamped into
/// `[EDGE_MARGIN, vw - EDGE_MARGIN - cw]` (falling back to flush against the
/// left edge if the content is wider than the viewport has room for at
/// all -- resizing to fit stays out of scope, matching the flip-only note
/// above). It is deliberately not mirrored into the CSS `@supports`
/// contract: no `position-try-fallbacks` keyword shifts along an axis the
/// way this function's clamp does (CSS's own analogous primitive,
/// `position-area`/`margin: auto`-driven "shift", is a materially larger
/// change that is not needed to close the gap on the devices concerned) --
/// so a genuinely CSS-Anchor-Positioning-conforming engine (Chromium among
/// them) is untouched by the clamp, exactly as the `matches()` early-return
/// above keeps it for flip. iOS Safari has no CSS Anchor Positioning
/// support at all, so it always takes this exact fallback path regardless.
/// `top-layer.spec.ts` Rule 12 is the oracle, reusing Rule 11's
/// own no-anchor-engine simulation (`stripAnchorSupportsBlock`,
/// `MOBILE_VIEWPORT`).
///
/// ## Vertical shift + size clamp (`dev-docs/backlog.md` row 10 residual
/// scope: "shift/size clamping")
///
/// The inline-axis shift directly above is the horizontal half. The block
/// axis has two related parts, built on the same construction rather than
/// a second positioning path:
///
/// 1. **Vertical position shift.** The inline-axis shift mirrored onto the
///    block axis: the side decision only ever swaps `side` to its own
///    axis's opposite (the `opposite` map above), so a
///    `side="left"`/`side="right"` placement's *vertical* position -- like
///    a `side="top"`/`side="bottom"` placement's *horizontal* one -- comes
///    entirely from `align`, computed once against the trigger in
///    `place()` above, and is not otherwise checked against the viewport.
///    It is clamped into `[EDGE_MARGIN, vh - EDGE_MARGIN - ch]`, the same
///    construction as the horizontal clamp above, `Math.max` for the
///    identical reason (a viewport too short for the content at all -- the
///    *top* edge wins here, matching "the left edge wins" for the
///    horizontal case; part 2 just below is what actually keeps that
///    overflow from reaching the screen, unlike the horizontal case, which
///    leaves it in place by design). A `side="top"`/`side="bottom"`
///    placement is never shifted along its main axis: see "Exclusion zone"
///    below.
///
/// 2. **Size clamp.** Unlike width (resizing stays out of scope -- see the
///    note on the flip-contract paragraph above, and this crate's own
///    per-component width caps), a too-*tall* overlay is the ordinary case
///    for every listbox/menu here (`Select`, `Combobox`, `DropdownMenu`,
///    `Menubar`) once its option/item count outgrows a short viewport --
///    simply letting it "run past the edge" the way the horizontal clamp
///    does would mean losing access to every option below the fold with no
///    way to reach it. So: when the content's own natural height (`ch`,
///    already reflecting whatever `max-height` a component's own
///    stylesheet applies -- `combobox/style.css`'s
///    `.dx-combobox-list` declares `max-height: 300px; overflow: hidden
///    auto;`, and `content.offsetHeight` is read *after* that CSS has
///    taken effect) exceeds the room of the chosen side (`avail.h`: the room
///    beside the anchor for a top/bottom placement, `vh - 2 * EDGE_MARGIN`
///    for left/right; `Math.max(0, ...)`-floored), this caps `content.style.
///    maxHeight` there and sets `overflowY: 'auto'` so the overlay's own
///    content scrolls instead of continuing to overflow the viewport edge.
///
///    Never fights a component's own sizing (this module's own governing
///    rule -- see [`ensure_anchor_positioning_styles`]'s doc): `ch` already
///    reflects any smaller cap a component's own stylesheet imposes, so
///    this branch only ever engages -- and only ever shrinks further --
///    when even that self-imposed cap still doesn't fit the room this
///    specific open actually has; a component whose own sizing already fits
///    never takes this branch at all. `box-sizing: border-box` is forced
///    alongside `maxHeight`: `ch` is `content.offsetHeight`, always the
///    *border-box* height regardless of the element's own `box-sizing`,
///    but `max-height` the CSS property is defined relative to `box-sizing`
///    -- under the initial `content-box` value, a `max-height` caps only
///    the content box, and padding/border then add back on *top* of that
///    for the rendered `offsetHeight`, silently under-clamping by however
///    much padding/border the element carries (the raw, unthemed
///    `primitives::popover::PopoverContent` of `top-layer.spec.ts` Rule 15's
///    fixture, which has no `.dx-popover-content` styling, overshoots
///    `availableHeight` by ~14px from the UA `[popover]` stylesheet's
///    default padding/border). See the `reposition()` code's own comment on
///    this line for the full account, including why this is a real gap for
///    any themed consumer that doesn't independently declare `border-box`
///    itself (or whose declaration doesn't reach the element), not an
///    artifact of that one fixture. Set as a plain inline style, unlike
///    the width contract (`--dx-anchor-width`, `dev-docs/backlog.md` row 47,
///    below) which publishes a custom property instead -- deliberately, not
///    an inconsistency: an inline style always wins the cascade over *any*
///    author-stylesheet declaration regardless of specificity (CSS
///    Cascading and Inheritance's origin/importance ordering), which is
///    required here and not merely convenient. `select/style.css`'s
///    `.dx-select-list` declares `overflow: visible`, by its own
///    comment "never scrolls (no max-height here) ... collision-aware ...
///    positioning belongs upstream" -- this *is* that upstream, and a real
///    class selector this crate's zero-specificity `:where(...)` engine
///    convention (`ensure_anchor_positioning_styles`'s doc) could never
///    out-specificity-fight without abandoning that convention for every
///    anchored overlay, not just this one -- exactly why the width
///    contract's own opt-in-via-`var()` approach is not reused here: this
///    clamp must apply whether or not a component asked for it, so it needs
///    the one thing in the cascade nothing but `!important` can beat.
///    Cleared, not merely left alone, once no longer needed: this function
///    re-runs on every tracked scroll/resize/`visualViewport` event for the
///    life of the open overlay ("Scroll/resize tracking" below), and a
///    viewport that grows back (an iOS keyboard closing, a window
///    un-shrinking) must let a component's own sizing govern again rather
///    than staying pinned to a stale cap from a smaller moment.
///
/// Both parts stay JS-fallback-only, deliberately, for the same reason the
/// inline-axis shift does (see that section's own reasoning): a
/// genuinely CSS-Anchor-Positioning-conforming engine (Chromium
/// among them) has the identical latent gap for *position* --
/// `position-try-fallbacks` declares no shift-along-an-axis primitive
/// either engine could use, nothing here is Firefox/WebKit-specific -- and
/// its `@supports` block gains no size-clamp coverage from CSS at all
/// either (no W3C-shipped primitive shrinks an anchored box to fit the way
/// Floating UI's `size()` middleware does in userland JS). That is a real,
/// acknowledged residual on the conforming engine: the clamps are scoped to
/// the JS-fallback gap `dev-docs/backlog.md` row 10 names, matching the
/// horizontal precedent rather than rewriting a working engine's code
/// path (a conforming engine's placement is still checked against the
/// exclusion zone below and handed to this path when it breaks it).
/// Oracle: `top-layer.spec.ts` Rule 15.
///
/// ## Exclusion zone (`dev-docs/backlog.md` row 210; it governs the main
/// axis, where "Vertical shift + size clamp" above does not apply)
///
/// The open Combobox list must never cover its own input (an iPhone with
/// the keyboard up is the case). Clamping the panel into the viewport along
/// the axis `side` names, which is the axis the anchor sits on, cannot work: with the
/// keyboard up the visual viewport is short, a flip that picks a side
/// without checking it fit, and a vertical clamp (`top >= EDGE_MARGIN`)
/// then drags the list ONTO the input; a size clamp measured against the
/// WHOLE viewport rather than the room beside the anchor could never shrink
/// the list far enough to avoid it. The CSS Anchor Positioning arm is not
/// safer on its own: trusting any placement that lands on a contract spot
/// (judged against the LAYOUT viewport, which a keyboard does not shrink)
/// caps nothing.
///
/// The construction, for every caller: **the anchor's box is an exclusion zone on
/// the main axis**, in visual-viewport coordinates.
///
/// 1. *Side by room.* `room[side]` is the space between the anchor's edge (plus the gap)
///    and the viewport edge (minus `EDGE_MARGIN`). The preferred side is kept while the
///    panel fits there, else the opposite side if it fits or simply has more room.
/// 2. *No main-axis shift.* Only the cross-axis clamp survives (sliding along the
///    anchor's edge, Floating UI's `shift` on the alignment axis); nothing moves the
///    panel across the anchor.
/// 3. *Size to the room.* A panel taller (wider, for left/right) than the chosen side's
///    room is capped to it and scrolls itself. The limits are published as
///    `--dx-available-height` / `--dx-available-width` (Base UI's `--available-height`)
///    on every call, on both arms, so a sheet can say
///    `max-height: min(18rem, var(--dx-available-height, 18rem))`
///    (`combobox/style.css` does). The panel's natural size is measured with those
///    caps lifted (in the same synchronous task, so nothing paints) and its scroll
///    offset restored.
/// 4. *The CSS arm is trusted only while it honours the zone.* A placement that
///    matches a contract spot is accepted only if the box is inside the visual viewport
///    and does not intersect the anchor; otherwise the JS path above takes over.
/// 5. *Growth while open.* A `ResizeObserver` on the panel re-places it when its
///    content changes size (typing in a Combobox), and a panel above the anchor is
///    pinned by its BOTTOM edge, so growth extends away from the anchor by
///    construction rather than by a re-measure.
///
/// Oracle: `playwright/combobox-placement.spec.ts` (geometry sweep over
/// viewports, input positions, schemes, typing and the list's own scroll).
///
/// ## Anchor width contract (`dev-docs/backlog.md` row 47)
///
/// A styled listbox whose stylesheet says `min-width: 100%` (as
/// `preview/src/components/select/style.css`'s `.dx-select-list` does) means
/// 100% of the *trigger* only while the list is `position: absolute` inside a
/// `position: relative` `Select` root; once promoted to the top layer via
/// `popover` its containing block is the viewport (see
/// [`position_anchor_style`]'s doc), so the same percentage means 100% of the
/// *viewport* and the listbox opens the full width of the viewport. A rule
/// `@supports (anchor-name: --a) { .dx-select-list[popover] { min-width:
/// anchor-size(width); } }` cannot restore trigger-relative sizing on the CSS
/// Anchor Positioning path: this crate's `#[css_module]` class-hashing does
/// not scope selectors written only inside an `@supports` body
/// (`dev-docs/issues/css-module-supports-scoping.md`; the same gap
/// [`ensure_anchor_positioning_styles`]'s doc describes for the shared
/// engine stylesheet -- the same gap, here in a *component's own*
/// stylesheet rather than a hand-copied engine snippet),
/// so the rule that ships targets the unhashed `.dx-select-list` while the
/// live element carries `.dx-select-list-<hash>`. And even where the
/// `@supports` route applied, it would only cover the
/// CSS-Anchor-Positioning path -- this function's own fallback
/// override above corrects *position* on every other engine, but never
/// touches `width`, so Firefox/WebKit (wherever this fallback
/// engages) would have no width fix regardless.
///
/// So the contract is engine-level, not a one-off CSS patch: this function's
/// own `reposition()` (the measurement step above, which -- unlike the
/// inline-position override -- runs unconditionally on *every*
/// engine, conforming or not) publishes the trigger's live width as the
/// custom property `--dx-anchor-width` on the content element, every time
/// it measures -- see the `content.style.setProperty('--dx-anchor-width',
/// ...)` call right after `t` is computed, deliberately placed *before*
/// the `matches()` early return so a conforming engine gets it too, and
/// kept fresh by the same scroll/resize/`visualViewport` tracking that
/// re-invokes this function for position. A themed stylesheet then
/// opts in with `min-width: var(--dx-anchor-width, 100%)` (`100%` fallback
/// keeps the native/non-web arm, where the list is still DOM-relative,
/// correct) -- a plain custom property, not a class the engine has any
/// opinion about, so this stays consistent with the rest of this module's
/// contract: the engine publishes *facts* (a trigger's current width) and
/// never bakes in a specific component's styling intent. The shared
/// `@supports` stylesheet (`ANCHOR_POSITIONING_CSS_JS_LITERAL` below) also
/// declares `--dx-anchor-width: anchor-size(width)` on the same marker
/// classes, as a CSS-native belt alongside this JS write's suspenders --
/// Chromium substitutes `anchor-size()` correctly when it is only
/// referenced from inside a custom-property declaration and then read back
/// through `var()` at a sizing property's use site, so it is worth keeping.
/// It is still only a belt, not the whole contract: this
/// JS write is what covers Firefox/WebKit and any engine that
/// reaches this fallback at all, and is authoritative regardless of what
/// the CSS path already set. Oracle: `select.spec.ts`'s
/// listbox-width assertion; `top-layer.spec.ts` Rule 13 (fit-content
/// width) runs on this file's own *unstyled* fixture and cannot see a
/// themed component's CSS, so it does not cover this contract -- see that
/// rule's own header doc.
///
/// ## Scroll/resize tracking
///
/// Tracking listeners install unconditionally for the life of every open
/// overlay: an anchored overlay (`ColorPicker`'s popup) would otherwise
/// float viewport-fixed while the page scrolls underneath it, detached from
/// its trigger. It is not gated on this function's own measurement having
/// judged CSS Anchor Positioning non-conforming at open time and taken over
/// with inline styles (the "neither matches" branch below) -- see "iOS
/// keyboard" further down for why. The function
/// tracks its own outcome in the `usingFallback` flag closed over by
/// `reposition()`; once a call sets it, every subsequent call -- including
/// ones driven by the listeners added below -- skips the `matches()` check
/// and unconditionally recomputes and re-applies the inline position. When
/// CSS Anchor Positioning is (still) placing the content correctly,
/// `usingFallback` never flips and `reposition()` keeps returning early
/// after one extra `getBoundingClientRect()` pair per event -- the
/// platform's own `anchor()` re-resolution does the real work there, this
/// is just confirming it still is.
///
/// Capture-phase (so scroll inside a nested scroll container is seen too --
/// `scroll` does not bubble, only capture reaches it from `window`),
/// passive `scroll` and a plain `resize` listener on `window` re-run the
/// same `reposition()` function, rAF-throttled (a `rafScheduled` flag drops
/// any additional event that fires before the next paint) so a fast scroll
/// gesture doesn't queue up more synchronous layout work than one
/// measurement per frame. The flip decision is re-evaluated on every
/// re-measure (`reposition()` always recomputes `primary`/`flipped` from
/// the current, live `getBoundingClientRect()`/viewport size), so a resize
/// that changes which side fits gets the same flip treatment the initial
/// placement does. All listeners are removed -- via the same
/// `dioxus.recv()`-gated teardown every other eval-channel hook in this
/// crate uses (see `use_popover_sync` above) -- when `open` goes false or
/// this component unmounts; nothing is left listening on `window` (or
/// `visualViewport`) past the overlay's own lifetime.
/// Deliberately narrow: no `ResizeObserver` on the trigger or content
/// elements for *position* -- only `window` (and `visualViewport`)
/// `scroll`/`resize`, which is all a *position* (not size) correction
/// needs. Re-running `reposition()` on every tracked event re-applies the
/// inline-axis shift (see that section above) exactly as it
/// re-applies flip -- both live inside the same function, so a scroll or
/// resize that changes how much room the trigger has re-clamps the same way
/// the initial placement does, with no separate wiring needed.
///
/// ## iOS keyboard
///
/// On iOS Safari the `Combobox` options list would render on top of its own
/// search input after focusing/typing, from two gaps, both handled here:
///
/// 1. **Visual-viewport awareness.** `reposition()` reading only
///    `window.innerWidth`/`innerHeight` plus `getBoundingClientRect()` would
///    miss that iOS Safari's on-screen keyboard shrinks and offsets
///    `window.visualViewport` while leaving `window.inner{Width,Height}`
///    completely unchanged -- so a keyboard-driven layout change would be
///    invisible to every fit/flip computation in this function, regardless
///    of whether anything re-ran it. `viewportMetrics()` below handles it:
///    every `vw`/`vh` read, and the trigger/content rects compared against
///    them, go through `window.visualViewport` (falling back to
///    `documentElement.client*` when absent, i.e. the plain layout viewport
///    on an engine without it) -- see that function's own doc for the
///    coordinate-space bookkeeping this requires (`position: fixed` stays
///    relative to the *layout* viewport even once the *visual* one is what
///    the fit decision needs to reason about). The size is
///    `min(visualViewport, documentElement.client*)` so a classic
///    scrollbar is never counted as room -- see [`VIEWPORT_METRICS_JS`].
/// 2. **Tracking regardless of how the open was placed.** Attaching the
///    scroll/resize listeners only inside the `if (usingFallback)` branch,
///    i.e. only when the very first `reposition()` call had already judged
///    CSS Anchor Positioning non-conforming, would give an overlay that CSS
///    Anchor Positioning placed correctly at *open* time no listener at all
///    for the rest of that open -- "an engine that gets here already
///    re-resolves `anchor()` on scroll for free" is true for scroll, but
///    silently assumes the set of contract-legal positions itself can never
///    change again for the life of that one open. It can: an iOS keyboard
///    appearing is exactly such a change, and Combobox is the one anchored
///    overlay whose opening reliably summons one (a focused text input).
///    So the listener installation sits outside that `if` entirely (see
///    below): tracking is unconditional for the life of every open overlay,
///    and `reposition()`'s own `matches()` check is what keeps a
///    still-conforming engine's re-checks a no-op.
///
/// Ruled out, so as not to be re-tried: neither gap is a
/// "plain no anchor support" problem -- `use_anchor_position_fallback`
/// self-corrects fully and immediately for an overlay with no
/// CSS-anchor support *from open*, byte-identical to native placement (see
/// "The flip contract" above); and a failed trigger lookup (the
/// `document.querySelector` returning `null`) lands the content off-screen
/// entirely, not on top of its own trigger -- neither symptom matches.
/// `playwright/oracle/tier2-html/top-layer.spec.ts` Rule 11 is the
/// oracle for both gaps, including the specific "conforming at
/// open, CSS Anchor Positioning support removed mid-open" shape gap 2 is
/// about (that file's own header doc records which of the seven
/// `use_anchor_position_fallback` consumers this sandbox's Chromium can
/// actually be driven into that exact shape for, and why the rest are
/// marked as needing a real device instead).
///
/// `anchor_id` must be the exact same string [`anchor_name_style`] was
/// built from for this overlay's trigger (every call site threads its
/// content's own id through both, so they always match by construction --
/// see `PopoverCtx::content_id`'s doc in `popover.rs` for the bug this
/// guards against for that component specifically). The trigger element
/// itself is found by matching that id's `anchor-name` back out of its
/// inline `style` attribute (a plain substring match -- cheap, and every
/// trigger always sets this style whether or not it ends up used) rather
/// than requiring a second id to be threaded through separately.
///
/// The query includes the closing `;` [`anchor_name_style`] always ends
/// with (`anchor-name: --dxa-{id};`), not just `--dxa-{id}` bare --
/// required, not cosmetic: this repo's
/// `use_unique_id` ids are plain incrementing integers rendered as decimal
/// strings, so an id like `dxc-4` is a literal string *prefix* of `dxc-40`,
/// `dxc-41`, etc. A page with several sibling anchored instances (the
/// `Combobox` page's demo variants, each contributing its own
/// `ComboboxInput`) hits this: without the trailing `;`, `document.querySelector`
/// for anchor id `dxc-4`'s substring pattern matches a *different* input's
/// `anchor-name: --dxa-dxc-40;` first in DOM order, measuring the wrong
/// trigger's position entirely and landing the listbox hundreds of pixels
/// off-screen (`combobox.spec.ts`'s "controlled value and controlled open
/// stay in sync" test could never reach an option).
/// The `;` is never itself ambiguous across ids: it is not part of any
/// digit sequence, so `dxc-4;` cannot appear inside `dxc-40;`.
#[cfg(feature = "web")]
pub(crate) fn use_anchor_position_fallback(
    id: String,
    anchor_id: String,
    open: impl Readable<Target = bool> + Copy + 'static,
    side: crate::ContentSide,
    align: crate::ContentAlign,
    gap_px: u32,
) {
    // Best-effort early install for repeat opens in this WASM instance --
    // once this has run once (from any overlay), it's a same-tick Rust-side
    // Cell check on every later call, nothing more. It is *not* what makes
    // the very first, cold-load open correct, though: see
    // `anchor_positioning_inject_js`'s doc for why that guarantee instead
    // comes from prepending the identical injection statement onto the
    // measurement eval's own script below.
    use_effect(ensure_anchor_positioning_styles);

    crate::js_listener::use_js_listeners(move || {
        if !open.cloned() {
            return None;
        }
        let id = id.clone();
        let anchor_id = anchor_id.clone();
        let side = side.as_str();
        let align = align.as_str();
        // Prepended into the same `document::eval()` script as the
        // measurement below -- see `anchor_positioning_inject_js`'s doc for
        // why this specific pairing (one script, not two separate `eval()`
        // calls) is what actually guarantees the stylesheet is in effect
        // before `reposition()`'s first `getBoundingClientRect()` call.
        let inject = anchor_positioning_inject_js();
        let viewport_metrics_js = VIEWPORT_METRICS_JS;
        let measure_js = MEASURE_AT_FULL_WIDTH_JS;
        // Synchronous up to and including the first `reposition()` call
        // below, not deferred to `requestAnimationFrame`: this runs from an
        // effect declared (and therefore, on Dioxus's normal mount-order
        // guarantee, run) after `use_popover_sync`'s own effect in every
        // call site, so `showPopover()` has already been dispatched -- and
        // `getBoundingClientRect()`/`offsetWidth` force a synchronous layout
        // in every engine regardless of paint timing, so there is nothing to
        // wait a frame for. This matters, not just simpler: wrapping the body
        // in `requestAnimationFrame(() => {{ ... }})` silently never runs
        // at all on a real Firefox build -- an eval's `document::eval` call
        // is a bare, uncaptured expression whose returned handle can be
        // dropped at the end of a synchronous closure with no ill effect
        // (the same fire-and-forget shape `use_popover_sync`'s own
        // synchronous `document::eval` calls use throughout this file), but
        // dropping it before a callback deferred to a later task/frame ever
        // fires appears to tear down the JS side before that callback runs.
        // A synchronous body has no such window -- which is exactly why,
        // below, the script *is* kept alive (a `JsListeners`, not a dropped
        // eval handle) for as long as tracking might need it: its
        // listeners are real pending callbacks.
        Some(crate::js_listener::JsListeners::install(&format!(
            r#"
            {inject}
            const content = document.getElementById('{id}');
            const trigger = document.querySelector(
                '[style*="anchor-name: --dxa-{anchor_id};"]'
            );
            const align = '{align}';
            const gap = {gap_px};
            const side = '{side}';
            // The only side `position-try-fallbacks: flip-block,
            // flip-inline` can ever swap this one to: top/bottom flip
            // on the block axis, left/right on the inline axis -- each
            // of the four sides has exactly one relevant flip axis, so
            // there is exactly one opposite candidate, not a set of
            // four.
            const opposite = {{ top: 'bottom', bottom: 'top', left: 'right', right: 'left' }}[side];

            // Sticky once flipped true by a `reposition()` call below --
            // see this function's doc, "Scroll/resize tracking": once this
            // engine's anchor-positioning integration has been judged
            // non-conforming, every later re-measure (scroll/resize-driven
            // or not) skips the `matches()` check and unconditionally
            // re-applies the inline position, rather than re-litigating
            // whether CSS anchoring "started working" mid-overlay.
            let usingFallback = false;

            // iOS keyboard handling (see this function's doc, "iOS
            // keyboard"): visual-viewport-aware metrics.
            // `window.innerWidth`/`innerHeight` do NOT shrink when iOS
            // Safari's on-screen keyboard appears; `window.visualViewport`'s
            // `width`/`height` do -- so every fit/flip decision below reads
            // through here instead of `window.inner*` directly (and the
            // helper, `VIEWPORT_METRICS_JS`, also excludes the classic
            // scrollbar via `documentElement.clientWidth/clientHeight`, which
            // `window.inner*` and some engines' `visualViewport` include),
            // and this is what makes placement keyboard-*aware* rather than
            // merely keyboard-*re-triggered* (gap 2, handled separately below,
            // is what makes it re-run at all). `offsetLeft`/`offsetTop` is
            // the visual viewport's own origin offset from the *layout*
            // viewport's (mainly a pinch-zoom-panning concept, included for
            // completeness): `getBoundingClientRect()` always reports
            // layout-viewport-relative coordinates regardless of which
            // viewport is visually showing, and the `position: fixed` this
            // function's override sets is likewise positioned relative to
            // the layout viewport, not the visual one -- so trigger/content
            // rects are shifted into visual-viewport space by this offset
            // before comparison, and shifted back out of it when writing
            // the final inline `top`/`left`. Falls back to
            // `documentElement.client` width/height and `0` on an engine
            // with no `visualViewport` at all. Defined once, shared with
            // `use_point_anchor_clamp`: see `VIEWPORT_METRICS_JS`.
            {viewport_metrics_js}
            {measure_js}

            // Exclusion-zone placement (backlog row 210, see this
            // function's doc, "Exclusion zone"): the anchor's box is a no-go
            // area for the panel on the MAIN axis (the one `side` names), in
            // visual-viewport coordinates, for every caller. EDGE_MARGIN is
            // the gap kept to the viewport edge.
            const EDGE_MARGIN = 4;
            const vertical = side === 'top' || side === 'bottom';

            // The panel's box where the engine placed it, without its own
            // running animations: an open recipe's `translate` (the theme's
            // 4px drop) or scale is already applied at the synchronous first
            // measurement, and would make a correct `anchor()` placement miss
            // `matches()` by its offset and hand a working CSS path to the JS
            // fallback (backlog row 378). Each finite animation is seeked to
            // its end for one read and put back in the same task, so nothing
            // paints and none of them finishes.
            function restingRect(el) {{
                const anims = el.getAnimations().filter((a) => a.currentTime !== null && a.effect
                    && Number.isFinite(a.effect.getComputedTiming().endTime));
                const saved = anims.map((a) => a.currentTime);
                for (const a of anims) a.currentTime = a.effect.getComputedTiming().endTime;
                const r = el.getBoundingClientRect();
                anims.forEach((a, i) => {{ a.currentTime = saved[i]; }});
                return r;
            }}

            function reposition() {{
                if (!content || !trigger) return;
                const t = trigger.getBoundingClientRect();
                // Engine contract (dev-docs/backlog.md row 47):
                // every anchored content's trigger width is published as
                // `--dx-anchor-width`, unconditionally -- not gated on
                // `usingFallback` below -- since this write needs to reach
                // a themed stylesheet's `min-width: var(--dx-anchor-width,
                // ...)` rule regardless of which positioning path this
                // engine takes.
                content.style.setProperty('--dx-anchor-width', t.width + 'px');
                // Measure the panel's NATURAL size: our own caps (the
                // published `--dx-available-*` properties a sheet may read in
                // its `max-height`, and the inline caps the JS path sets
                // below) feed back into the very size the side decision
                // reasons about, so they are dropped, measured around, and
                // written again in this same synchronous task (no frame
                // paints the uncapped state). The panel's own scroll offset
                // is restored because removing a cap can clamp it.
                const keepScroll = content.scrollTop;
                content.style.removeProperty('--dx-available-height');
                content.style.removeProperty('--dx-available-width');
                if (usingFallback) {{
                    content.style.maxHeight = '';
                    content.style.maxWidth = '';
                    content.style.overflowY = '';
                    content.style.overflowX = '';
                    content.style.boxSizing = '';
                }}
                const c = restingRect(content);
                // `let`: re-measured at full width below once this call has
                // committed to overriding the position (`measureAtFullWidth`).
                let cw = content.offsetWidth;
                let ch = content.offsetHeight;
                const vp = viewportMetrics();
                const vw = vp.width;
                const vh = vp.height;
                // See `viewportMetrics()`'s doc just above: re-express the
                // rects `getBoundingClientRect()` gave in layout-viewport
                // coordinates as visual-viewport-relative ones, so they
                // compare like-for-like against `vw`/`vh` in every formula
                // below.
                const tTop = t.top - vp.offsetTop;
                const tBottom = t.bottom - vp.offsetTop;
                const tLeft = t.left - vp.offsetLeft;
                const tRight = t.right - vp.offsetLeft;
                const cTop = c.top - vp.offsetTop;
                const cLeft = c.left - vp.offsetLeft;

                // Room on each side of the anchor, in visual-viewport space:
                // from the anchor's edge (plus the gap) to the viewport edge
                // (minus the margin). This is the space a panel on that side
                // may occupy WITHOUT touching the anchor.
                const room = {{
                    bottom: vh - EDGE_MARGIN - (tBottom + gap),
                    top: tTop - gap - EDGE_MARGIN,
                    right: vw - EDGE_MARGIN - (tRight + gap),
                    left: tLeft - gap - EDGE_MARGIN,
                }};

                // The side to use: the preferred one while the panel fits
                // there, else the opposite one if it fits there or simply has
                // more room (Floating UI's flip with a preferred-side bias).
                // Never anything but the two sides of the `side` axis.
                function chooseSide() {{
                    const fits = (s) => (vertical ? ch <= room[s] : cw <= room[s]);
                    if (fits(side)) return side;
                    return (fits(opposite) || room[opposite] > room[side]) ? opposite : side;
                }}
                let chosen = chooseSide();

                // The size limits the chosen side imposes, published as
                // `--dx-available-height` / `--dx-available-width` (Base UI's
                // `--available-height`) so a sheet can say
                // `max-height: min(18rem, var(--dx-available-height, 18rem))`.
                // On the main axis it is the room beside the anchor; on the
                // cross axis it is the whole viewport minus the margins.
                function availableFor(s) {{
                    return vertical
                        ? {{ h: Math.max(0, room[s]), w: Math.max(0, vw - 2 * EDGE_MARGIN) }}
                        : {{ h: Math.max(0, vh - 2 * EDGE_MARGIN), w: Math.max(0, room[s]) }};
                }}
                function publish(a) {{
                    content.style.setProperty('--dx-available-height', a.h + 'px');
                    content.style.setProperty('--dx-available-width', a.w + 'px');
                }}

                // Same trigger-relative formula the `[data-side]`/
                // `[data-align]` CSS contract promises, parameterised over
                // *which* side is currently being placed. Operates entirely
                // in visual-viewport space (the `t*` locals above).
                // `align` is logical: under RTL the inline start edge is the
                // right one (the panel inherits `direction` from its trigger).
                const rtl = getComputedStyle(content).direction === 'rtl';
                const hAlign = rtl && align === 'start' ? 'end' : (rtl && align === 'end' ? 'start' : align);
                function place(effectiveSide) {{
                    let top;
                    let left;
                    if (effectiveSide === 'top') {{
                        top = tTop - gap - ch;
                    }} else if (effectiveSide === 'bottom') {{
                        top = tBottom + gap;
                    }} else if (align === 'start') {{
                        top = tTop;
                    }} else if (align === 'end') {{
                        top = tBottom - ch;
                    }} else {{
                        top = tTop + (tBottom - tTop) / 2 - ch / 2;
                    }}
                    if (effectiveSide === 'left') {{
                        left = tLeft - gap - cw;
                    }} else if (effectiveSide === 'right') {{
                        left = tRight + gap;
                    }} else if (hAlign === 'start') {{
                        left = tLeft;
                    }} else if (hAlign === 'end') {{
                        left = tRight - cw;
                    }} else {{
                        left = tLeft + (tRight - tLeft) / 2 - cw / 2;
                    }}
                    return {{ top, left }};
                }}

                // Placement contract (published at EVERY exit below, CSS-anchor path and JS
                // fallback alike, so a stylesheet never re-measures or guesses): the side the
                // panel actually sits on (`data-placed-side`, which differs from the requested
                // `data-side` after a flip) and the anchor's centre in the panel's own space
                // (`--dx-anchor-offset-x/-y`, px from the panel's left/top edge; physical, so
                // right for RTL too). `left`/`top` are the panel's final position in the same
                // visual-viewport space as `tLeft`/`tTop`.
                function publishPlacement(placedSide, left, top) {{
                    const r1 = (n) => Math.round(n * 10) / 10;
                    content.setAttribute('data-placed-side', placedSide);
                    content.style.setProperty('--dx-anchor-offset-x', r1(tLeft + (tRight - tLeft) / 2 - left) + 'px');
                    content.style.setProperty('--dx-anchor-offset-y', r1(tTop + (tBottom - tTop) / 2 - top) + 'px');
                }}

                const primary = place(side);
                const flipped = place(opposite);

                // Tolerance for rounding/subpixel noise -- see this
                // function's doc for why this checks the *outcome* rather
                // than trusting a feature-detection flag.
                function matches(pos) {{
                    return Math.abs(cTop - pos.top) <= 2 && Math.abs(cLeft - pos.left) <= 2;
                }}

                if (!usingFallback && (matches(primary) || matches(flipped))) {{
                    // A working anchor-positioning engine placed this at one
                    // of the two contract-legal spots. It is only trusted
                    // while that placement honours the exclusion zone: the
                    // engine judges "fits" against the LAYOUT viewport (which
                    // an on-screen keyboard does not shrink) and caps nothing,
                    // so a box it placed may reach under the keyboard or off
                    // the visual viewport. Either violation (box outside the
                    // visual viewport, or box intersecting the anchor) hands
                    // this call to the JS path below, which is the authority.
                    const cBottom = cTop + ch;
                    const cRight = cLeft + cw;
                    const outside = cTop < -1 || cLeft < -1 || cBottom > vh + 1 || cRight > vw + 1;
                    const overlaps = cLeft < tRight - 1 && cRight > tLeft + 1 && cTop < tBottom - 1 && cBottom > tTop + 1;
                    if (!outside && !overlaps) {{
                        const cssSide = matches(primary) ? side : opposite;
                        publish(availableFor(cssSide));
                        publishPlacement(cssSide, cLeft, cTop);
                        return;
                    }}
                }}

                // Neither matches, or the engine's placement broke the
                // exclusion zone (or a previous call already committed to
                // this branch -- see `usingFallback` above): make the side
                // decision from plain viewport math. Sticky.
                usingFallback = true;
                // Measure-then-move class (see `MEASURE_AT_FULL_WIDTH_JS`):
                // re-measure at the viewport origin (full available width)
                // and redo the side decision with the true size; the final
                // write below is in this same synchronous block, so the
                // probe never paints.
                content.style.position = 'fixed';
                content.style.margin = '0';
                const full = measureAtFullWidth(content, vp);
                cw = full.width;
                ch = full.height;
                chosen = chooseSide();
                const avail = availableFor(chosen);

                // Size clamp: the panel shrinks (and scrolls itself) to the
                // room of the chosen side instead of overlapping the anchor
                // or leaving the viewport. `box-sizing: border-box` is forced
                // alongside the caps because `ch`/`cw` are border-box
                // `offset*` sizes while `max-height` is measured against
                // `box-sizing`; an inline style always wins the cascade over
                // a component's own rule, which is what a clamp that must
                // apply to every consumer needs.
                if (ch > avail.h || cw > avail.w) {{
                    content.style.boxSizing = 'border-box';
                }}
                if (ch > avail.h) {{
                    content.style.maxHeight = avail.h + 'px';
                    content.style.overflowY = 'auto';
                }}
                if (cw > avail.w && !vertical) {{
                    // Width is capped only when it is the MAIN axis (a
                    // left/right placement): a top/bottom panel wider than the
                    // viewport keeps the "left edge wins" rule.
                    content.style.maxWidth = avail.w + 'px';
                    content.style.overflowX = 'auto';
                }}
                publish(avail);
                ch = content.offsetHeight;
                cw = content.offsetWidth;

                let target = place(chosen);

                // Cross-axis shift (= Floating UI's `shift` on the
                // alignment axis): the panel may slide ALONG the anchor's
                // edge to stay on screen, never across it. top/bottom shift
                // horizontally, left/right vertically. The main axis is
                // deliberately NOT clamped: a clamp there would drag a
                // list onto its own input.
                if (vertical) {{
                    if (target.left < EDGE_MARGIN) {{
                        target = {{ top: target.top, left: EDGE_MARGIN }};
                    }} else if (target.left + cw > vw - EDGE_MARGIN) {{
                        // `Math.max`: on a viewport too narrow for the content
                        // the left edge wins.
                        target = {{ top: target.top, left: Math.max(EDGE_MARGIN, vw - EDGE_MARGIN - cw) }};
                    }}
                }} else if (target.top < EDGE_MARGIN) {{
                    target = {{ top: EDGE_MARGIN, left: target.left }};
                }} else if (target.top + ch > vh - EDGE_MARGIN) {{
                    target = {{ top: Math.max(EDGE_MARGIN, vh - EDGE_MARGIN - ch), left: target.left }};
                }}

                content.style.position = 'fixed';
                content.style.margin = '0';
                content.style.inset = 'auto';
                content.style.transform = 'none';
                content.style.setProperty('position-try-fallbacks', 'none');
                // Shift back out of visual-viewport space (see
                // `viewportMetrics()`'s doc above): `position: fixed` is
                // relative to the layout viewport. A panel above the anchor is
                // pinned by its BOTTOM edge, so content that grows while open
                // (typing in a Combobox) extends upward, away from the anchor,
                // by construction rather than by a re-measure.
                if (chosen === 'top') {{
                    content.style.top = 'auto';
                    content.style.bottom = (vp.cbBottom - (target.top + ch + vp.offsetTop)) + 'px';
                }} else {{
                    content.style.bottom = 'auto';
                    content.style.top = (target.top + vp.offsetTop) + 'px';
                }}
                content.style.left = (target.left + vp.offsetLeft) + 'px';
                publishPlacement(chosen, target.left, target.top);
                if (content.scrollTop !== keepScroll) content.scrollTop = keepScroll;
            }}

            reposition();

            // Settle loop, deferred across animation frames, *only* once the
            // synchronous call just above has already committed to the
            // fallback (`usingFallback`): the synchronous call above can run
            // *before* this content's own children have finished growing into
            // their final size (`date-picker.spec.ts`'s "anchors the popup"
            // case) -- `DatePickerPopover`'s calendar grid, populated
            // by a nested component's own reactive render, still
            // measures an intermediate 352x178px box (narrower-grid/
            // shorter-before-rows placeholder shape) at the exact tick this
            // effect runs, rather than its final 276x278px, throwing off
            // `place()`'s `cw`/`ch`-dependent math (here, the align="center"
            // horizontal centering) by however much the box still has left
            // to grow -- 38px off center on that box's ~76px of subsequent
            // width growth. A **content-size** race, not a scroll/layout-
            // timing one: `getBoundingClientRect()`/`offsetWidth` already
            // force a synchronous style/layout recalc (this function's doc
            // above), so the geometry read back is never stale relative to
            // the DOM as it stood at that instant -- the DOM itself just
            // wasn't finished changing shape yet. A single extra
            // `requestAnimationFrame` call is not enough: it still observes
            // the pre-grow 352px box even several real frames and 300ms
            // later, since whatever later render pass grows the grid does
            // not land within one, or even a handful of, animation frames of
            // this effect's own -- so instead of guessing a frame count, this
            // polls `content.offsetWidth`/`offsetHeight` once per frame and
            // keeps re-running `reposition()` until two consecutive frames
            // read the *same* box size (the settle signal: nothing is still
            // resizing), capped at `MAX_SETTLE_FRAMES` frames as a backstop
            // against a pathological page that never stabilizes.
            //
            // Gated on `usingFallback` -- *not* run unconditionally -- for a
            // reason `top-layer.spec.ts` Rule 8's ColorPicker case pins (an
            // unconditional loop regresses it): once the first call has
            // judged CSS Anchor Positioning conforming, every extra
            // `reposition()` call in a naive unconditional loop re-runs the
            // *same* `matches()` decision from scratch, and each of those
            // re-checks is one more chance for one transient frame --
            // mid-fade-in-animation, mid this-same content-growth window,
            // anything that momentarily moves the content's painted rect
            // off the anchor-computed one by more than the 2px tolerance --
            // to be wrongly read as "the engine failed," latching
            // `usingFallback` true *permanently* (it is deliberately sticky,
            // this function's doc above) for a case that was never actually
            // broken. Only entering this loop when the first call already
            // committed to the fallback removes that risk entirely: from
            // that point on every `reposition()` call skips the `matches()`
            // check outright (`usingFallback` short-circuits the `if` below)
            // and unconditionally re-applies fresh inline geometry -- the
            // decision to accept the fallback's placement is never
            // relitigated, only its geometry is refined. A CSS-Anchor-
            // Positioning engine that placed this correctly on the first
            // call is instead left completely alone by this loop.
            if (usingFallback) {{
                const MAX_SETTLE_FRAMES = 30; // ~0.5s at 60fps
                let lastW = content.offsetWidth;
                let lastH = content.offsetHeight;
                let stableStreak = 0;
                let frame = 0;
                const settle = () => {{
                    frame++;
                    reposition();
                    const w = content.offsetWidth;
                    const h = content.offsetHeight;
                    stableStreak = (w === lastW && h === lastH) ? stableStreak + 1 : 0;
                    lastW = w;
                    lastH = h;
                    if (stableStreak < 2 && frame < MAX_SETTLE_FRAMES) {{
                        requestAnimationFrame(settle);
                    }}
                }};
                requestAnimationFrame(settle);
            }}

            // iOS keyboard, gap 2 -- see this function's doc,
            // "iOS keyboard": tracking listeners install
            // UNCONDITIONALLY, for the life of every open overlay, not only
            // once `usingFallback` is already `true` at this point in the
            // script. An overlay that CSS Anchor Positioning placed
            // correctly at open time still needs them: "an
            // engine that gets here already re-resolves `anchor()` on
            // scroll for free" is true for scroll, but silently
            // assumes the *set of contract-legal positions* itself never
            // changes for the rest of that open. An iOS Safari on-screen
            // keyboard breaks exactly that assumption mid-open (shrinking
            // `visualViewport` out from under an already-placed overlay),
            // and Combobox is the one anchored overlay whose opening
            // reliably summons a keyboard (a focused text input).
            // `reposition()` itself
            // still only overrides when `matches()` fails (or `usingFallback`
            // is already sticky-true), so a still-conforming CSS-anchor
            // engine re-running `reposition()` on every scroll/resize costs
            // one extra `getBoundingClientRect()` pair and returns
            // immediately -- no behavior change for the common case,
            // only for the one this tracking exists to close.
            //
            // `scroll` is added on `window` with `capture: true` because
            // the event does not bubble -- capture on `window` is the only
            // way to observe a scroll on a nested scroll container, not
            // just the document itself. `visualViewport`'s own `resize`
            // (fires when the keyboard opens/closes, or on pinch-zoom) and
            // `scroll` (fires when the visual viewport pans, e.g. while a
            // focused input is scrolled into view above the keyboard) are
            // added too, when present, alongside the existing `window`
            // listeners -- `window`'s own `resize` does not fire for a
            // visual-viewport-only change, since `window.inner` width/
            // height do not change when the keyboard appears (this
            // function's doc, gap 1). rAF-throttled: `rafScheduled`
            // collapses any number of scroll/resize events firing before
            // the next paint (from either source) into a single
            // `reposition()` call. Registered through `listen`, so they are
            // removed on close/unmount exactly as every other eval-channel
            // listener in this file (`JsListeners`' derived teardown).
            let rafScheduled = false;
            const onTrack = () => {{
                if (rafScheduled) return;
                rafScheduled = true;
                requestAnimationFrame(() => {{
                    rafScheduled = false;
                    reposition();
                }});
            }};
            // A scroll INSIDE the panel (the user scrolling the list) cannot
            // move the anchor, and `reposition()` itself briefly lifts the
            // panel's size cap (which can nudge its scroll offset): reacting
            // to it would be a pointless layout pass per scroll frame and
            // could feed back on itself.
            const onScroll = (e) => {{
                if (e.target !== document && content && content.contains(e.target)) return;
                onTrack();
            }};
            listen(window, 'scroll', onScroll, {{ capture: true, passive: true }});
            listen(window, 'resize', onTrack, {{ passive: true }});
            // An anchor inside an ANIMATING ancestor (a submenu's trigger inside its root menu's
            // `dx-pop-in`, scale .95 -> 1) is measured at the animation's current scale by
            // `getBoundingClientRect()`, so a panel opened inside that window is placed against a
            // stale, shrunken box (rtl.spec's Sub "opens to the LEFT" covers the ~1px overlap
            // with its trigger). Re-place once the ancestor's animation ends: the anchor is then at rest.
            const onAnimEnd = (e) => {{
                if (trigger && e.target !== content && e.target.contains && e.target.contains(trigger)) onTrack();
            }};
            listen(document, 'animationend', onAnimEnd, {{ capture: true, passive: true }});
            const vv = window.visualViewport;
            listen(vv, 'resize', onTrack, {{ passive: true }});
            listen(vv, 'scroll', onTrack, {{ passive: true }});
            // Content that grows or shrinks while open (typing in a Combobox
            // changes the filtered list) is re-placed in the same frame the
            // browser lays it out, so a list opened upward never extends over
            // its anchor and one opened downward re-checks its room.
            const resizeObserver = typeof ResizeObserver === 'function' && content
                ? new ResizeObserver(onTrack)
                : null;
            if (resizeObserver) {{
                resizeObserver.observe(content);
                cleanup(() => resizeObserver.disconnect());
            }}
            // No cleanup of data-placed-side / --dx-anchor-offset-*: a superseded run's teardown
            // resolves AFTER its successor's first (synchronous) publish and would erase it; the
            // next open republishes before paint, and a closed panel is display:none.
            "#
        )))
    });
}

// No native (Blitz) counterpart: every call site
// (`tooltip.rs`/`hover_card.rs`/`popover.rs`) is itself inside a
// `#[cfg(feature = "web")]`-only component -- the modal/non-modal
// and mount/unmount boundaries in those files are real child-component
// boundaries specifically so the *hook call* can differ per arm without a
// conditional-hook-call hazard (see each file's comment), so a same-named
// no-op stub here would never be called and is dead code by construction,
// like `PopoverKind` and `position_anchor_style` above.
