//! Shared viewport metrics, close-on-scroll and the point-anchor clamp.

#[cfg_attr(not(feature = "web"), allow(unused_imports))]
use super::*;

/// The ONE viewport measurement every JS collision/clamp script in this
/// crate shares -- defines `viewportMetrics()` inside whatever `eval` string
/// it is interpolated into ([`use_anchor_position_fallback`]'s
/// `reposition()` and [`use_point_anchor_clamp`]). Any other script
/// that needs "how much room is there" must splice this in rather than read
/// `window.inner{Width,Height}` or `visualViewport.{width,height}` itself
/// (`viewport_metrics_js_is_the_only_viewport_read` below fails the build
/// otherwise).
///
/// **Why not `window.innerWidth`/`visualViewport.width` alone.** What
/// content can occupy is the viewport *minus the classic scrollbar* -- a
/// 15px gutter on a desktop browser with always-on scrollbars, on both
/// axes. `window.innerWidth/innerHeight` always include it, and engines
/// disagree on whether `visualViewport.width/height` do (Chromium
/// excludes it; a clamp that read the scrollbar-inclusive value would land
/// an overlay `vw - 4` = ~11px past the content area).
/// `document.documentElement.clientWidth/clientHeight` is the layout
/// viewport with scrollbars excluded, by definition, in every engine.
///
/// **Why still `visualViewport`.** It is what shrinks when iOS Safari's
/// on-screen keyboard appears (while `clientHeight` and `innerHeight` do not)
/// and under pinch-zoom (`width = clientWidth / scale`) -- the iOS keyboard
/// handling (`dev-docs/backlog.md` row 28) depends on that. So the
/// usable size is the *smaller* of the two on each axis:
/// `min(visualViewport.size, documentElement.client size)`. Keyboard or
/// zoom: the visual viewport is smaller and wins.
/// Scrollbar on an engine whose `visualViewport` ignores it: `client*` is
/// smaller and wins. Where both agree (this Chromium): the same value.
/// `offsetLeft`/`offsetTop` stay purely `visualViewport`'s (the pan origin
/// the callers shift rects by); `0` without it. `|| innerWidth` guards the
/// quirks-mode/detached-root case where `clientWidth` reads `0`.
///
/// **Why also the fixed containing block.** The panels are `position: fixed`,
/// so the box they are placed in is the fixed containing block, which is NOT
/// always `clientWidth`: with `html { scrollbar-gutter: stable }` this
/// Chromium reports `clientWidth` 390 while the containing block (and
/// `html.getBoundingClientRect().width`) is 375, so a `clientWidth` clamp would leave
/// wide panels (the multi-month date picker) 11-15px past the right edge at
/// 390/600/900px. The clamp box is therefore also measured from a cached
/// `position: fixed; inset: 0` probe (one invisible node, created on first
/// use and re-attached if the page removes it; reading its rect is the same
/// single layout read `clientWidth` costs), and the smallest of the three wins.
///
/// Spliced into the scripts as a function declaration, so it hoists and may
/// be called anywhere in them.
#[cfg_attr(not(feature = "web"), allow(dead_code))]
pub(crate) const VIEWPORT_METRICS_JS: &str = r#"
            function viewportMetrics() {
                const vv = window.visualViewport;
                const root = document.documentElement;
                // The box a `position: fixed` panel is placed in: a cached,
                // invisible `fixed; inset: 0` probe. `clientWidth` is NOT that
                // box under `scrollbar-gutter: stable` (390 vs 375 in this
                // Chromium), so it is only ever the upper bound below.
                let probe = window.__dxFixedCb;
                if (!probe || !probe.isConnected) {
                    probe = document.createElement("div");
                    probe.setAttribute("aria-hidden", "true");
                    probe.setAttribute("data-dx-fixed-cb", "");
                    probe.style.cssText = "position:fixed;inset:0;visibility:hidden;pointer-events:none;";
                    root.appendChild(probe);
                    window.__dxFixedCb = probe;
                }
                const cb = probe.getBoundingClientRect();
                const clientW = Math.min(root.clientWidth || window.innerWidth, cb.width || Infinity);
                const clientH = Math.min(root.clientHeight || window.innerHeight, cb.height || Infinity);
                return {
                    width: vv ? Math.min(vv.width, clientW) : clientW,
                    height: vv ? Math.min(vv.height, clientH) : clientH,
                    offsetLeft: vv ? vv.offsetLeft : 0,
                    offsetTop: vv ? vv.offsetTop : 0,
                    // The fixed containing block's own bottom edge: what a
                    // panel pinned by `bottom:` is measured from.
                    cbBottom: cb.bottom,
                };
            }
"#;

// ---------------------------------------------------------------------------
// Close a non-modal anchored panel when the page scrolls away from it
// (backlog row 193, item 7 of `dev-docs/research/next-round-plan-2026-10-14.md`).
// ---------------------------------------------------------------------------

/// What makes [`use_close_on_scroll`] close a panel.
#[cfg(feature = "web")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScrollClose {
    /// Any document / scroll-container scroll closes it (Tooltip, HoverCard, menus and Select
    /// when they do not lock the page). These hold nothing a user would mind losing.
    AnyScroll,
    /// Only the ANCHOR leaving the viewport closes it (Popover and the DatePicker / ColorPicker
    /// popovers built on it, Combobox): they can hold half-entered input, and a small scroll that
    /// keeps the anchor in view must not throw it away.
    AnchorLeavesViewport,
}

#[cfg(feature = "web")]
impl ScrollClose {
    const fn as_str(self) -> &'static str {
        match self {
            Self::AnyScroll => "any",
            Self::AnchorLeavesViewport => "anchor",
        }
    }
}

/// The script behind [`use_close_on_scroll`]. Parameters, in order: the panel's id, the anchor's
/// id (`""` for a point-anchored panel such as a context menu), the policy string.
///
/// Every listener goes through `listen` (`js_listener.rs`), so it is removed when the hook's
/// scope ends or `active` goes false, and the scroll listener is `passive`
/// (`scripts/check-blocking-scroll-listeners.sh`). The observer is released through `cleanup`.
///
/// **The mobile keyboard caveat.** Opening the virtual keyboard resizes and pans the VISUAL
/// viewport and makes the browser scroll the focused field into view; if that counted as "the
/// page scrolled", a Combobox would close the moment it is tapped. By construction it cannot:
///  * nothing listens to `visualViewport` scroll/resize as a close signal -- they only extend a
///    quiet window;
///  * a scroll within `QUIET_MS` of a focus change into the panel or the anchor, a key press in
///    them, or a visual-viewport resize/pan is ignored (the browser's focus-driven
///    scroll-into-view lands in that window);
///  * the `AnchorLeavesViewport` policy ignores scrolling altogether and watches the anchor with
///    an `IntersectionObserver`: a focus-driven scroll reveals the field, it never hides it. If the
///    anchor goes out of view inside a quiet window (a keyboard that resized the viewport before
///    the browser scrolled the field back), the decision waits for the window to end.
///
/// A scroll INSIDE the panel (the user scrolling a long list) or inside the anchor (an `<input>`
/// scrolling its own text) is never a reason to close.
#[cfg(feature = "web")]
const CLOSE_ON_SCROLL_JS: &str = r#"
    const id = await dioxus.recv();
    const anchorId = await dioxus.recv();
    const policy = await dioxus.recv();
    const QUIET_MS = 300;
    const content = document.getElementById(id);
    const trigger = anchorId
        ? document.querySelector('[style*="anchor-name: --dxa-' + anchorId + ';"]')
        : null;
    let sent = false;
    const close = () => {
        if (sent) return;
        sent = true;
        dioxus.send(true);
    };
    // Opening a panel focuses things and may scroll them into view.
    let quietUntil = performance.now() + QUIET_MS;
    const quiet = () => { quietUntil = performance.now() + QUIET_MS; };
    const mine = (node) =>
        node instanceof Node &&
        ((content && content.contains(node)) || (trigger && trigger.contains(node)));
    listen(document, 'focusin', (e) => { if (mine(e.target)) quiet(); }, true);
    listen(document, 'keydown', (e) => { if (mine(e.target)) quiet(); }, true);
    const vv = window.visualViewport;
    if (vv) {
        listen(vv, 'resize', quiet, { passive: true });
        listen(vv, 'scroll', quiet, { passive: true });
    }
    if (policy === 'any') {
        // `window` + capture: `scroll` does not bubble, so this is the only way to see a nested
        // scroll container. Document and scroll-container scrolls only -- never the visual viewport.
        listen(window, 'scroll', (e) => {
            if (mine(e.target)) return;
            if (performance.now() < quietUntil) return;
            close();
        }, { capture: true, passive: true });
    } else if (trigger && typeof IntersectionObserver === 'function') {
        // Closes on the in-view -> out-of-view transition only: an anchor that is already out of
        // view when the panel opens is not "leaving". An anchor that goes out of view inside a
        // quiet window (a keyboard resizing the viewport before the browser has scrolled the
        // focused field back into it) is looked at again once the window is over; re-observing
        // delivers a fresh notification with the then-current state.
        let seen = false;
        let timer = 0;
        const observer = new IntersectionObserver((entries) => {
            for (const entry of entries) {
                if (entry.isIntersecting) {
                    seen = true;
                } else if (seen) {
                    const wait = quietUntil - performance.now();
                    if (wait <= 0) {
                        close();
                    } else if (!timer) {
                        timer = setTimeout(() => {
                            timer = 0;
                            observer.unobserve(trigger);
                            observer.observe(trigger);
                        }, wait + 50);
                    }
                }
            }
        });
        observer.observe(trigger);
        cleanup(() => { clearTimeout(timer); observer.disconnect(); });
    }
"#;

/// Closes the panel `content_id` (anchored to `anchor_id`; `""` for a point-anchored one) by
/// calling `set_open(false)` according to `policy`, for as long as `active` is true. The one
/// helper every non-modal anchored primitive calls, so the policy cannot drift per component;
/// modal / scroll-locking surfaces do not call it (they keep their `ScrollLockGuard`).
/// See [`CLOSE_ON_SCROLL_JS`] for the mobile-keyboard caveat.
#[cfg(feature = "web")]
pub(crate) fn use_close_on_scroll(
    content_id: String,
    anchor_id: String,
    active: impl Readable<Target = bool> + Copy + 'static,
    policy: ScrollClose,
    set_open: Callback<bool>,
) {
    crate::js_listener::use_js_listeners(move || {
        if !active.cloned() {
            return None;
        }
        Some(
            crate::js_listener::JsListeners::install(CLOSE_ON_SCROLL_JS)
                .arg(content_id.clone())
                .arg(anchor_id.clone())
                .arg(policy.as_str())
                .on_message(move |_: bool| set_open.call(false)),
        )
    });
}

/// The ONE way a clamp script measures a panel it is about to move:
/// defines `measureAtFullWidth(content, vp)` (returns `{ width, height }`,
/// the border-box `offsetWidth`/`offsetHeight`), spliced in next to
/// [`VIEWPORT_METRICS_JS`].
///
/// **The class this guards (measure-then-move on a shrink-to-fit box).** A
/// `position: fixed` panel with `width: auto` is sized by shrink-to-fit:
/// `min(max-content, room between its `left` and the containing block's right
/// edge)`. So the width a clamp reads depends on the very position the clamp is
/// about to replace. A context menu opened 2px from the right edge would measure its
/// squeezed ~148px instead of its real 160px max-content width, be placed for
/// 148 (`vw - 4 - 148`), then grow back to 160 at that `left` and end flush
/// with the edge instead of `EDGE_MARGIN` away from it. It stays hidden only
/// while a large `min-width` happens to exceed the content.
///
/// **The construction.** Park the panel at the viewport origin (`inset: auto`,
/// `left`/`top` at the visual viewport's own origin, where it has the widest
/// room it can ever have) and read there, so the measured size is the size it
/// will have anywhere the clamp can put it. The caller writes its final
/// `left`/`top` in the same synchronous block, so no frame paints the probe
/// position. Sizes stay `offsetWidth`/`offsetHeight` (layout sizes, unaffected
/// by an open animation's `scale`), never `getBoundingClientRect()`.
#[cfg_attr(not(feature = "web"), allow(dead_code))]
pub(crate) const MEASURE_AT_FULL_WIDTH_JS: &str = r#"
            function measureAtFullWidth(content, vp) {
                content.style.inset = 'auto';
                content.style.left = vp.offsetLeft + 'px';
                content.style.top = vp.offsetTop + 'px';
                return { width: content.offsetWidth, height: content.offsetHeight };
            }
"#;

/// Clamps a **point-anchored** overlay -- one positioned at a raw `(x, y)`
/// coordinate with no anchor element at all -- into the viewport.
/// `dev-docs/backlog.md` row 10, item 5.2: the one piece of that row's
/// collision-detection scope [`use_anchor_position_fallback`] above cannot
/// cover, because that function is keyed on an *anchor element*: it resolves
/// a trigger via `document.querySelector('[style*="anchor-name: ..."]')` and
/// every one of its flip/shift/size-clamp branches reasons in terms of that
/// trigger's own rect (`place()`'s `side`/`align` formulas) and a CSS
/// Anchor Positioning conformance check it can defer to
/// (`matches()`/`usingFallback`). `ContextMenu`'s content has none of that
/// -- it opens at a raw click/long-press point with no persistent trigger
/// element to anchor off of at all (see
/// `context_menu::ContextMenuContentRendered`'s own doc for why `anchor-
/// name`/`position-anchor` are never wired up for it) -- so there is no
/// anchor rect to measure, no flip to compute, and no conforming-engine
/// case to defer to (a raw point is never something the platform's
/// `anchor()` could have placed in the first place). This hook is the
/// "virtual-anchor" construction that row explicitly calls for: the point
/// itself stands in for the anchor.
///
/// **A parallel, smaller construction, not a branch inside
/// [`use_anchor_position_fallback`]**: that function's script is already a
/// long, heavily cross-referenced sequence (flip
/// decision, inline/vertical shift, size clamp, `--dx-anchor-width`
/// publication, scroll/resize/`visualViewport` tracking, a multi-frame
/// settle loop) built and tested entirely around resolving and measuring an
/// anchor element. Threading a no-anchor special case through it would mean
/// either an anchor-shaped no-op path through every one of those stages
/// (`place()` has nothing to compute from, `matches()` has nothing to
/// compare against, `usingFallback` protects a conformance check that never
/// applies here) or an early branch that skips most of the function's body
/// for this one caller -- both add real risk to that function's own
/// passing oracle coverage (`top-layer.spec.ts` rules 5-7, 9, 11-15) for a
/// case that shares only the final clamp *arithmetic*, not the surrounding
/// machinery. This hook instead reuses just that shared arithmetic --
/// the same `EDGE_MARGIN` (4px) convention, the same `Math.max` viewport-
/// too-small-to-fit-at-all fallback, the same `border-box`-forced max-height
/// clamp -- as its own small, independent script, mirroring
/// [`use_anchor_position_fallback`]'s "Vertical shift + size clamp"
/// doc section's construction rather than its code.
///
/// `x`/`y` are `ContextMenuTrigger`'s own already visual-viewport-corrected
/// click coordinates (see that file's `visual_viewport_offset` doc) --
/// i.e. already layout-viewport-relative, exactly matching this content's
/// `position: fixed; left/top` inline styles. Re-expressed here relative to
/// `visualViewport`'s own origin before clamping (mirroring
/// [`use_anchor_position_fallback`]'s `viewportMetrics()`/`tTop`/`tLeft`
/// treatment) so the point being clamped and the bounds it's clamped
/// against are in the same coordinate space, then shifted back before the
/// final inline-style write, since `position: fixed` is relative to the
/// *layout* viewport, not the visual one.
///
/// Measured and clamped once, synchronously, right after the popover is
/// promoted to the top layer -- must be called after [`use_popover_sync`]'s
/// own effect at the call site, the same ordering requirement
/// [`use_anchor_position_fallback`] documents for itself (Dioxus's normal
/// same-component effect-ordering guarantee: this hook's own `use_effect`
/// call must be the later one). Unlike that function, no scroll/resize
/// tracking and no multi-frame settle loop: `ContextMenuContent`'s items
/// are ordinary, synchronously-rendered children (not
/// `DatePickerPopover`'s asynchronously-growing calendar grid --
/// [`use_anchor_position_fallback`]'s own "settle loop" doc), and the menu
/// already suppresses page scroll while open
/// (`ContextMenuContentRendered`'s own wheel/touchmove effect) rather than
/// following it, so there is nothing this hook would need to re-measure
/// against after the initial open.
#[cfg(feature = "web")]
pub(crate) fn use_point_anchor_clamp(
    id: String,
    x: i32,
    y: i32,
    open: impl Readable<Target = bool> + Copy + 'static,
) {
    use_effect(move || {
        if !open.cloned() {
            return;
        }
        let id = id.clone();
        let _ = document::eval(&format!(
            r#"
            {viewport_metrics_js}
            {measure_js}
            const content = document.getElementById('{id}');
            if (content) {{
                const vp = viewportMetrics();
                const vw = vp.width;
                const vh = vp.height;
                const offsetLeft = vp.offsetLeft;
                const offsetTop = vp.offsetTop;
                // Measured at full available width, NOT at the click point:
                // a context menu opened near the right edge is a shrink-to-
                // fit box with almost no room there, so `offsetWidth` at the
                // click point is its squeezed width, and the clamp below would
                // place it for a width it no longer has once moved. The final
                // `left`/`top` writes at the end of this block land before any
                // paint, so the probe position is never shown.
                const full = measureAtFullWidth(content, vp);
                const cw = full.width;
                const ch = full.height;

                // Same EDGE_MARGIN convention as `use_anchor_position_fallback`'s
                // own shift clamp (`dev-docs/backlog.md` row 10) -- see this
                // function's own doc for the full construction this mirrors.
                // `x`/`y` re-expressed relative to `visualViewport`'s own
                // origin so the point and the bounds it's clamped against
                // share one coordinate space; shifted back before the final
                // write below.
                const EDGE_MARGIN = 4;
                let left = {x} - offsetLeft;
                let top = {y} - offsetTop;
                if (left < EDGE_MARGIN) {{
                    left = EDGE_MARGIN;
                }} else if (left + cw > vw - EDGE_MARGIN) {{
                    // `Math.max`: on a viewport too narrow for the content
                    // at all, the left edge wins rather than the two
                    // branches disagreeing about which edge to honor --
                    // matches `use_anchor_position_fallback`'s identical
                    // horizontal clamp exactly (width stays out of scope,
                    // same as there).
                    left = Math.max(EDGE_MARGIN, vw - EDGE_MARGIN - cw);
                }}
                if (top < EDGE_MARGIN) {{
                    top = EDGE_MARGIN;
                }} else if (top + ch > vh - EDGE_MARGIN) {{
                    top = Math.max(EDGE_MARGIN, vh - EDGE_MARGIN - ch);
                }}

                // Size clamp: same construction as `use_anchor_position_
                // fallback`'s own gap 2 ("Vertical shift + size clamp")
                // -- once even a fully top-clamped position
                // still can't fit the content in the viewport's remaining
                // height, shrink instead of shifting further, and let the
                // menu's own items scroll instead of running off the
                // bottom of the screen. `box-sizing: border-box` forced
                // alongside `maxHeight` for the identical reason that
                // function's own doc gives (an inline style, so it always
                // wins the cascade over whatever a themed menu stylesheet
                // does or doesn't declare for `box-sizing`).
                const availableHeight = Math.max(0, vh - 2 * EDGE_MARGIN);
                if (ch > availableHeight) {{
                    content.style.boxSizing = 'border-box';
                    content.style.maxHeight = availableHeight + 'px';
                    content.style.overflowY = 'auto';
                }} else {{
                    content.style.maxHeight = '';
                    content.style.overflowY = '';
                    content.style.boxSizing = '';
                }}

                content.style.left = (left + offsetLeft) + 'px';
                content.style.top = (top + offsetTop) + 'px';
            }}
            "#,
            id = id,
            x = x,
            y = y,
            viewport_metrics_js = VIEWPORT_METRICS_JS,
            measure_js = MEASURE_AT_FULL_WIDTH_JS,
        ));
    });
}

// No native (Blitz) counterpart: `ContextMenuContentRendered`'s native
// (Blitz) arm (`context_menu.rs`) renders the raw click point with no
// clamping at all -- the same functional
// floor every other web-only construction in this file leaves that arm at
// (see the identical note just above, for `use_anchor_position_fallback`).
// Blitz has no popover-API/top-layer support to promote the content into in
// the first place, so this is consistent with the rest of that arm rather
// than a new gap.
