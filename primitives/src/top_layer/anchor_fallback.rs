//! The placement script behind every anchored overlay (`use_anchor_position_fallback`): rule P of
//! backlog row 477.

#[cfg_attr(
    not(feature = "web"),
    expect(unused_imports, reason = "only the web build uses it")
)]
use super::*;

/// What a panel is placed against.
#[cfg(feature = "web")]
pub(super) enum Anchor {
    /// The trigger whose `anchor-name` carries this id (`anchor_name_style`).
    Element(String),
    /// A document point `(pageX, pageY)`, such as the press that opened a context menu.
    Point(i32, i32),
}

/// The placement script. Arguments, read as one array: the panel's id, the anchor id (`""` for a
/// point), the point or `null`, the preferred side, the alignment and the gap in px.
///
/// Everything is in document coordinates, so the frame a browser reports rects in (visual viewport
/// in iOS WebKit, layout viewport in Chromium) never matters, and the page carries the panel when
/// it scrolls. The panel is a top-layer popover with `position: absolute; inset: auto; margin: 0;
/// top: 0; left: 0`, moved only by `transform`; a panel above or left of its anchor adds
/// `translateY(-100%)` / `translateX(-100%)`, so its own size is never needed to pin it.
///
/// A CSS-anchored panel (`anchor.css`) is left alone while it sits on one of its two contract
/// spots, inside the visible region and clear of the anchor; anything else hands the panel to this
/// script for the rest of the open. On every frame of a scroll, resize or visual-viewport event it
/// chooses the side (the preferred one unless it lacks the natural size and the other has more
/// room, with a 24px hysteresis), recomputes the transform, and writes only the properties whose
/// value changed: a flip, a layout shift or a nested scroller, never a page scroll. The size cap
/// (`max-height`, `max-width` on a left or right side) follows the room on the chosen side live.
///
/// Published for stylesheets: `--dx-anchor-width`, `--dx-available-height/-width`,
/// `data-placed-side` and `--dx-anchor-offset-x/-y` (the anchor's centre in the panel's own space).
#[cfg(feature = "web")]
const PLACE_JS: &str = r#"
const [id, anchorId, point, SIDE, ALIGN, GAP] = await dioxus.recv();
const content = document.getElementById(id);
const trigger = point ? null : document.querySelector('[style*="anchor-name: --dxa-' + anchorId + ';"]');
if (!content || (!point && !trigger)) return;
const M = 4;
const V = SIDE === 'top' || SIDE === 'bottom';
const OPP = { top: 'bottom', bottom: 'top', left: 'right', right: 'left' }[SIDE];
const cs = getComputedStyle(content);
const rtl = cs.direction === 'rtl';
const ownH = parseFloat(cs.maxHeight) || Infinity;
const ownW = parseFloat(cs.maxWidth) || Infinity;
const put = writer(content);
let origin = [50, 0];
let placed = false;
let side = null;

// Hands the panel to this script: absolute in the top layer, moved by `transform` alone.
function take() {
    placed = true;
    origin = originOf(content);
    put('position', 'absolute');
    put('inset', 'auto');
    put('margin', '0');
    put('top', '0');
    put('left', '0');
    put('position-try-fallbacks', 'none');
}

// The panel's box without its running animations (an open recipe's translate or scale).
function restingRect() {
    const anims = content.getAnimations().filter((a) => a.currentTime !== null && a.effect
        && Number.isFinite(a.effect.getComputedTiming().endTime));
    const saved = anims.map((a) => a.currentTime);
    for (const a of anims) a.currentTime = a.effect.getComputedTiming().endTime;
    const r = content.getBoundingClientRect();
    anims.forEach((a, i) => { a.currentTime = saved[i]; });
    return r;
}

function reposition() {
    const v = visible();
    const a = pt ? { l: pt[0], r: pt[0], t: pt[1], b: pt[1] } : docRect(trigger, v);
    const cx = (a.l + a.r) / 2, cy = (a.t + a.b) / 2;
    let cw = content.offsetWidth, ch = content.offsetHeight;
    const room = (s) => ({ bottom: v.b - a.b - GAP - M, top: a.t - GAP - v.t - M, right: v.r - a.r - GAP - M, left: a.l - GAP - v.l - M })[s];
    const at = (s) => ({
        x: s === 'left' ? a.l - GAP - cw : s === 'right' ? a.r + GAP
            : ALIGN === 'center' ? cx - cw / 2 : (ALIGN === 'start') !== rtl ? a.l : a.r - cw,
        y: s === 'top' ? a.t - GAP - ch : s === 'bottom' ? a.b + GAP
            : ALIGN === 'center' ? cy - ch / 2 : ALIGN === 'start' ? a.t : a.b - ch,
    });
    const publish = (s, left, top) => {
        const r1 = (n) => Math.round(n * 10) / 10;
        if (content.getAttribute('data-placed-side') !== s) content.setAttribute('data-placed-side', s);
        put('--dx-anchor-width', a.r - a.l + 'px');
        put('--dx-anchor-offset-x', r1(cx - left) + 'px');
        put('--dx-anchor-offset-y', r1(cy - top) + 'px');
        put('--dx-available-height', Math.max(0, V ? room(s) : v.b - v.t - 2 * M) + 'px');
        put('--dx-available-width', Math.max(0, V ? v.r - v.l - 2 * M : room(s)) + 'px');
    };

    if (!placed) {
        const r = restingRect();
        const c = { l: r.left - v.ox, t: r.top - v.oy };
        const hit = (s) => Math.abs(c.t - at(s).y) <= 2 && Math.abs(c.l - at(s).x) <= 2;
        const s = hit(SIDE) ? SIDE : hit(OPP) ? OPP : null;
        const inside = c.t >= v.t - 1 && c.l >= v.l - 1 && c.t + ch <= v.b + 1 && c.l + cw <= v.r + 1;
        const clear = c.l + cw <= a.l + 1 || c.l >= a.r - 1 || c.t + ch <= a.t + 1 || c.t >= a.b - 1;
        if (s && inside && clear) { publish(s, c.l, c.t); return; }
        take();
        cw = content.offsetWidth;
        ch = content.offsetHeight;
    }

    const natH = content.scrollHeight + ch - content.clientHeight;
    const natW = content.scrollWidth + cw - content.clientWidth;
    const nat = V ? Math.min(natH, ownH) : Math.min(natW, ownW);
    const sp = room(SIDE), so = room(OPP);
    side = !side ? (sp >= nat || sp >= so ? SIDE : OPP)
        : side === SIDE ? (sp < nat && so > sp + 24 ? OPP : SIDE)
        : (sp >= nat + 24 || sp > so + 24 ? SIDE : OPP);
    let { x, y } = at(side);
    if (pt && V && x + cw > v.r - M) x = a.l - cw;
    if (V) x = Math.max(v.l + M, Math.min(x, v.r - M - cw));
    else y = Math.max(v.t + M, Math.min(y, v.b - M - ch));
    setPlacement(put, snap(side === 'left' ? a.l - GAP : x), snap(side === 'top' ? a.t - GAP : y),
        side === 'top', side === 'left', origin);

    const hc = Math.round(Math.max(V ? room(side) : v.b - v.t - 2 * M, 0));
    const wc = Math.round(Math.max(V ? v.r - v.l - 2 * M : room(side), 0));
    const tall = hc < Math.min(natH, ownH);
    const wide = wc < Math.min(natW, ownW);
    put('max-height', tall ? hc + 'px' : '');
    put('overflow-y', tall ? 'auto' : '');
    put('max-width', wide ? wc + 'px' : '');
    put('overflow-x', wide ? 'auto' : '');
    put('box-sizing', tall || wide ? 'border-box' : '');
    publish(side, side === 'left' ? a.l - GAP - content.offsetWidth : x,
        side === 'top' ? a.t - GAP - content.offsetHeight : y);
}

let queued = false;
const track = () => {
    if (queued) return;
    queued = true;
    requestAnimationFrame(() => { queued = false; reposition(); });
};
let pt = null;
if (point) {
    // A point opened too close to an edge sits at the edge margin instead.
    const v = visible();
    pt = [Math.min(Math.max(point[0], v.l + M), v.r - M), Math.min(Math.max(point[1], v.t + M), v.b - M)];
    take();
}
reposition();
listen(window, 'scroll', (e) => { if (e.target === document || !content.contains(e.target)) track(); }, { capture: true, passive: true });
listen(window, 'resize', track, { passive: true });
listen(document, 'animationend', (e) => { if (trigger && e.target !== content && e.target.contains && e.target.contains(trigger)) track(); }, { capture: true, passive: true });
listen(window.visualViewport, 'resize', track, { passive: true });
listen(window.visualViewport, 'scroll', track, { passive: true });
if (typeof ResizeObserver === 'function') {
    const observer = new ResizeObserver(track);
    observer.observe(content);
    cleanup(() => observer.disconnect());
}
"#;

/// Places the panel `id` against `anchor` for as long as `open` is true (see [`PLACE_JS`]).
#[cfg(feature = "web")]
pub(super) fn use_placement(
    id: String,
    anchor: Anchor,
    open: impl Readable<Target = bool> + Copy + 'static,
    side: crate::ContentSide,
    align: crate::ContentAlign,
    gap_px: u32,
) {
    crate::js_listener::use_js_listeners(move || {
        if !open.cloned() {
            return None;
        }
        let (anchor_id, point) = match &anchor {
            Anchor::Element(anchor_id) => (anchor_id.clone(), None),
            Anchor::Point(x, y) => (String::new(), Some((*x, *y))),
        };
        // The stylesheet insertion rides in the same script, so the first `getBoundingClientRect()`
        // already sees the `anchor()` rules (`anchor_positioning_inject_js`).
        let script = format!(
            "{}{}{}",
            anchor_positioning_inject_js(),
            PLACEMENT_HELPERS_JS,
            PLACE_JS
        );
        Some(crate::js_listener::JsListeners::install(&script).arg((
            id.clone(),
            anchor_id,
            point,
            side.as_str(),
            align.as_str(),
            gap_px,
        )))
    });
}

/// Places an anchored overlay next to its trigger (`anchor_id` is the id
/// [`anchor_name_style`] was built from; every call site threads the content's own id through
/// both). Call it after the overlay's `use_popover_sync` effect, so the panel is already in the top
/// layer when the first placement reads it.
///
/// A conforming CSS Anchor Positioning engine places the panel natively (`anchor.css`); this
/// measures the outcome instead of trusting `CSS.supports`, because an engine can parse
/// `anchor-name` and still fail to resolve `anchor()` against a popover. The script takes the
/// panel over when the painted position matches neither contract spot (the preferred side, and
/// the same formula on the opposite side that `position-try-fallbacks: flip-block, flip-inline`
/// can paint), lies outside the visible region (a keyboard shrinks it while the CSS engine still
/// judges against the layout viewport) or covers the anchor.
#[cfg(feature = "web")]
pub(crate) fn use_anchor_position_fallback(
    id: String,
    anchor_id: String,
    open: impl Readable<Target = bool> + Copy + 'static,
    side: crate::ContentSide,
    align: crate::ContentAlign,
    gap_px: u32,
) {
    use_effect(ensure_anchor_positioning_styles);
    use_placement(id, Anchor::Element(anchor_id), open, side, align, gap_px);
}

// No native (Blitz) counterpart: every call site is inside a `#[cfg(feature = "web")]`-only
// component, so a same-named no-op stub would be dead code.
