//! The geometry helpers every placement script shares, the point-anchored placement and the
//! tooltip's close-on-scroll.

#[cfg_attr(
    not(feature = "web"),
    expect(unused_imports, reason = "only the web build uses it")
)]
use super::*;

/// The ONE geometry the placement scripts share: `visible()`, `docRect(el, v)` and `writer(el)`,
/// spliced in front of `use_anchor_position_fallback`'s script and the item-aligned Select's
/// (`viewport_metrics_js_is_the_only_viewport_read` fails the build when another script reads the
/// viewport itself).
///
/// **Document coordinates.** `visible()` is the region the user can see, in document coordinates,
/// and `docRect()` an element's box in the same frame: a rect minus the origin of
/// `document.documentElement.getBoundingClientRect()`. `getBoundingClientRect()` is relative to the
/// visual viewport in iOS WebKit and to the layout viewport in Chromium, so adding scroll or
/// `visualViewport` offsets to a rect is right in one engine and wrong in the other; subtracting
/// the document's own origin is right in both. A panel written as `position: absolute` in the top
/// layer lives in these coordinates (its containing block is the initial containing block), so
/// the page carries it when it scrolls and nothing tracks it.
///
/// **The visible region.** A cached, invisible `position: fixed; inset: 0` probe gives the fixed
/// containing block (which is not `clientWidth` under `scrollbar-gutter: stable`) and its origin;
/// the visual viewport's offset and size narrow it to what a keyboard or a pinch leaves visible
/// (`visualViewport.offsetTop` is the visual viewport's top from the layout viewport's in every
/// engine).
///
/// **`writer(el)`** returns `put(property, value)`, which sets an inline property only when it
/// differs from the last value `put` wrote, so a frame that changes nothing writes nothing.
#[cfg_attr(
    all(not(feature = "web"), not(test)),
    expect(dead_code, reason = "only the web build and the tests use it")
)]
pub(crate) const PLACEMENT_HELPERS_JS: &str = r#"
function visible() {
    const vv = window.visualViewport;
    const root = document.documentElement;
    let probe = window.__dxFixedCb;
    if (!probe || !probe.isConnected) {
        probe = document.createElement('div');
        probe.setAttribute('aria-hidden', 'true');
        probe.setAttribute('data-dx-fixed-cb', '');
        probe.style.cssText = 'position:fixed;inset:0;visibility:hidden;pointer-events:none;';
        root.appendChild(probe);
        window.__dxFixedCb = probe;
    }
    const p = probe.getBoundingClientRect();
    const h = root.getBoundingClientRect();
    const l = p.left + (vv ? vv.offsetLeft : 0) - h.left;
    const t = p.top + (vv ? vv.offsetTop : 0) - h.top;
    return {
        l, t, ox: h.left, oy: h.top,
        r: l + Math.min(vv ? vv.width : Infinity, p.width),
        b: t + Math.min(vv ? vv.height : Infinity, p.height),
    };
}
function docRect(el, v) {
    const r = el.getBoundingClientRect();
    return { l: r.left - v.ox, t: r.top - v.oy, r: r.right - v.ox, b: r.bottom - v.oy };
}
function writer(el) {
    const memo = {};
    return (prop, value) => {
        if (memo[prop] === value) return;
        memo[prop] = value;
        el.style.setProperty(prop, value);
    };
}
"#;

/// What makes [`use_close_on_scroll`] close a panel.
#[cfg(feature = "web")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScrollClose {
    /// Any document or scroll-container scroll closes it. Only a Tooltip (and the surfaces that
    /// still ask for it) closes on scroll: an anchored panel otherwise stays open and scrollable,
    /// because the page carries it (`PLACEMENT_HELPERS_JS`).
    AnyScroll,
}

/// The script behind [`use_close_on_scroll`]. Parameters, in order: the panel's id and the anchor's
/// id (`""` for a point-anchored panel).
///
/// Every listener goes through `listen` (`js_listener.rs`), so it is removed when the hook's scope
/// ends or `active` goes false, and the scroll listener is `passive`
/// (`scripts/check-blocking-scroll-listeners.sh`).
///
/// **The mobile keyboard caveat.** Opening the virtual keyboard resizes and pans the VISUAL
/// viewport and makes the browser scroll the focused field into view; if that counted as "the
/// page scrolled", a panel would close the moment its field is tapped. By construction it cannot:
/// nothing listens to `visualViewport` scroll/resize as a close signal (they only extend a quiet
/// window), and a scroll within `QUIET_MS` of a focus change into the panel or the anchor, a key
/// press in them, or a visual-viewport resize or pan is ignored. A scroll INSIDE the panel or the
/// anchor is never a reason to close.
#[cfg(feature = "web")]
const CLOSE_ON_SCROLL_JS: &str = r#"
const id = await dioxus.recv();
const anchorId = await dioxus.recv();
const QUIET_MS = 300;
const content = document.getElementById(id);
const trigger = anchorId
    ? document.querySelector('[style*="anchor-name: --dxa-' + anchorId + ';"]')
    : null;
let sent = false;
let quietUntil = performance.now() + QUIET_MS;
const quiet = () => { quietUntil = performance.now() + QUIET_MS; };
const mine = (node) =>
    node instanceof Node &&
    ((content && content.contains(node)) || (trigger && trigger.contains(node)));
listen(document, 'focusin', (e) => { if (mine(e.target)) quiet(); }, true);
listen(document, 'keydown', (e) => { if (mine(e.target)) quiet(); }, true);
listen(window.visualViewport, 'resize', quiet, { passive: true });
listen(window.visualViewport, 'scroll', quiet, { passive: true });
listen(window, 'scroll', (e) => {
    if (sent || mine(e.target) || performance.now() < quietUntil) return;
    sent = true;
    dioxus.send(true);
}, { capture: true, passive: true });
"#;

/// Closes the panel `content_id` (anchored to `anchor_id`; `""` for a point-anchored one) by
/// calling `set_open(false)` when the page scrolls, for as long as `active` is true. Only a Tooltip
/// asks for it; see [`CLOSE_ON_SCROLL_JS`] for the mobile-keyboard caveat.
#[cfg(feature = "web")]
pub(crate) fn use_close_on_scroll(
    content_id: String,
    anchor_id: String,
    active: impl Readable<Target = bool> + Copy + 'static,
    policy: ScrollClose,
    set_open: Callback<bool>,
) {
    let ScrollClose::AnyScroll = policy;
    crate::js_listener::use_js_listeners(move || {
        if !active.cloned() {
            return None;
        }
        Some(
            crate::js_listener::JsListeners::install(CLOSE_ON_SCROLL_JS)
                .arg(content_id.clone())
                .arg(anchor_id.clone())
                .on_message(move |_: bool| set_open.call(false)),
        )
    });
}

/// Places a **point-anchored** overlay, one opened at a document point with no anchor element (a
/// context menu), with the same script as an anchored one: the point `(x, y)` is a zero-size
/// anchor. `x`/`y` are `pageX`/`pageY`, so the menu stays where the page put it when the page
/// scrolls. The corner of the menu sits at the point; a menu that lacks room below opens upward
/// (`translateY(-100%)`) and one that lacks room to the right opens leftward, then the cross axis
/// is clamped into the visible region.
///
/// Call it after [`use_popover_sync`]'s effect at the call site, so the panel is already in the top
/// layer when the first placement reads it. The point is read when the calling component mounts;
/// `ContextMenuContentRendered` mounts once per open.
#[cfg(feature = "web")]
pub(crate) fn use_point_anchor_placement(
    id: String,
    x: i32,
    y: i32,
    open: impl Readable<Target = bool> + Copy + 'static,
) {
    use_placement(
        id,
        Anchor::Point(x, y),
        open,
        crate::ContentSide::Bottom,
        crate::ContentAlign::Start,
        0,
    );
}

// No native (Blitz) counterpart: the native arm of `ContextMenuContentRendered` renders the raw
// point, and Blitz has no popover API to promote the content into the top layer.
