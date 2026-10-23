//! The DOM bridge (backlog row 154, layer 3): the one place where a windowed component meets
//! the page. `VirtualList` (windowed mode), `VirtualGrid`, `VirtualWrapGrid` and
//! `VirtualJustifiedGrid` all consume it.
//!
//! [`use_virtual_bridge`] gives a component, by construction:
//!
//! - **one passive `scroll` listener** on its container, plus a `window` `resize` listener and
//!   **one `ResizeObserver`** on the container, all registered through `js_listener.rs`'s
//!   `listen` / `cleanup`, so they are removed with the component's scope (row 159) and none can
//!   leak or block scrolling;
//! - **one report shape** ([`ScrollReport`]: `scrollTop`, `|scrollLeft|` so RTL reads like LTR,
//!   the visible size capped at the window's, and a scrolling flag that drops 600 ms after the
//!   last scroll event, which is when the frozen canvas size is released), deduplicated in the
//!   page so an unchanged state never crosses to Rust;
//! - **a render-signal write only when the window changes**: the engine lives in a non-reactive
//!   [`CopyValue`], and [`Bridge::publish`] bumps the one reactive revision only when the engine's
//!   [`Windowing::needs_render`] says a render would draw something different (row 143's
//!   quantised range, now for every consumer);
//! - for one-axis engines, the shared flows on top: a scroll report with its anchoring
//!   correction ([`Bridge::scroll_axis`]), a measured item ([`Bridge::measure`]) and
//!   `scroll_to(index)` ([`Bridge::reveal`]), each writing the corrected `scrollTop` back.
//!
//! What stays per component: what an engine is (one axis, or a [`super::GridEngine`] of two),
//! the keyboard model, focus moves and the DOM each one draws.

use dioxus::prelude::*;

use super::grid::GridEngine;
use super::justified_engine::JustifiedEngine;
use super::types::ScrollReport;
use super::virtualizer::Engine;
use crate::js_listener::{use_js_listeners, JsListeners};

/// An engine the bridge can drive: it says whether a render now would draw a different window.
pub(crate) trait Windowing {
    /// Whether rendering now would draw something different from what the last render (the
    /// engine's `plan`) drew.
    fn needs_render(&mut self) -> bool;
}

impl Windowing for Engine {
    fn needs_render(&mut self) -> bool {
        Engine::needs_render(self)
    }
}

impl Windowing for GridEngine {
    fn needs_render(&mut self) -> bool {
        GridEngine::needs_render(self)
    }
}

impl Windowing for JustifiedEngine {
    fn needs_render(&mut self) -> bool {
        JustifiedEngine::needs_render(self)
    }
}

/// A component's handle on its engine and its container. `Copy`, so event handlers capture it.
pub(crate) struct Bridge<E: 'static> {
    engine: CopyValue<E>,
    rev: Signal<u64>,
    container_id: Signal<String>,
}

impl<E: 'static> Clone for Bridge<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: 'static> Copy for Bridge<E> {}

/// Creates the engine (`init`, once), installs the container's scroll bridge and calls
/// `on_report` for every scroll report. `container_id` is the id the component renders on the
/// element that scrolls. The listeners are removed when the component unmounts.
pub(crate) fn use_virtual_bridge<E: Windowing + 'static>(
    container_id: Signal<String>,
    init: impl FnOnce() -> E,
    on_report: impl FnMut(Bridge<E>, ScrollReport) + Clone + 'static,
) -> Bridge<E> {
    let engine = use_hook(|| CopyValue::new(init()));
    let rev = use_signal(|| 0u64);
    let bridge = Bridge {
        engine,
        rev,
        container_id,
    };
    use_js_listeners(move || {
        let mut on_report = on_report.clone();
        Some(
            JsListeners::install(SCROLL_BRIDGE_JS)
                .arg(container_id.cloned())
                .on_message(move |report: ScrollReport| on_report(bridge, report)),
        )
    });
    bridge
}

impl<E: Windowing + 'static> Bridge<E> {
    /// The engine, for the render's `sync` / `plan` and for handlers.
    pub(crate) fn engine(self) -> CopyValue<E> {
        self.engine
    }

    /// Re-render the component if, and only if, the engine says the window changed.
    pub(crate) fn publish(self) {
        let (mut engine, mut rev) = (self.engine, self.rev);
        if engine.with_mut(|engine| engine.needs_render()) {
            let next = rev.peek().wrapping_add(1);
            rev.set(next);
        }
    }

    /// Subscribe the calling render to [`publish`](Self::publish). Call it in the render body.
    pub(crate) fn track(self) {
        let _ = self.rev.read();
    }

    /// Move the container's scroll position.
    pub(crate) fn set_scroll_top(self, top: u32) {
        set_scroll_top(self.container_id.peek().clone(), top);
    }
}

impl Bridge<Engine> {
    /// The one-axis flow for a scroll report: the engine reads the block axis, and when rows
    /// measured above the viewport moved the content, the corrected offset is written back to
    /// the container (and to the engine, so the next report agrees).
    pub(crate) fn scroll_axis(self, report: ScrollReport) {
        let mut engine = self.engine;
        let correction =
            engine.with_mut(|engine| engine.scroll(report.top, report.height, report.is_scrolling));
        if let Some(delta) = correction {
            let top = corrected(report.top, delta);
            self.set_scroll_top(top);
            engine.with_mut(|engine| engine.set_scroll_offset(top));
        }
        self.publish();
    }

    /// Item `index` measured `measured` px along the axis; a size change above the viewport
    /// moves the scroll position by the same amount, so what the reader sees stays put.
    pub(crate) fn measure(self, index: usize, measured: u32) {
        let mut engine = self.engine;
        let (adjustment, current) =
            engine.with_mut(|engine| (engine.resize_item(index, measured), engine.scroll_offset()));
        self.publish();
        if let Some(delta) = adjustment {
            self.set_scroll_top(corrected(current, delta));
        }
    }

    /// `scroll_to(index)`: centre item `index` in the viewport (clamped to the scroll range),
    /// which mounts it. Nothing past the end.
    pub(crate) fn reveal(self, index: usize) {
        let mut engine = self.engine;
        let target = engine.with_mut(|engine| {
            let target = engine.scroll_target(index)?;
            engine.set_scroll_offset(target);
            Some(target)
        });
        if let Some(target) = target {
            self.publish();
            self.set_scroll_top(target);
        }
    }
}

/// `offset` moved by a signed correction, saturating at both ends of the `u32` range.
pub(crate) fn corrected(offset: u32, delta: i32) -> u32 {
    offset.saturating_add_signed(delta)
}

/// A measured size in whole px, at least 1 (a collapsed item still occupies one index slot).
/// Rounded UP (ignoring float noise below 1/100 px): the canvas is the sum of these, so a
/// rounded-down size made the canvas shorter than the rows it holds and the container gained a
/// 1px scroll overflow (three 66.1875px rows measured 66 each: canvas 198, content 198.56).
pub(crate) fn measured_px(size: f64) -> u32 {
    if size.is_nan() {
        return 1;
    }
    (size - 0.01).max(1.0).ceil().min(f64::from(u32::MAX)) as u32
}

/// Moves a container's scroll position by id.
fn set_scroll_top(container_id: String, top: u32) {
    let eval = document::eval(
        r#"
        const [id, top] = await dioxus.recv();
        const container = document.getElementById(id);
        if (container) container.scrollTop = top;
        "#,
    );
    let _ = eval.send((container_id, top));
}

/// Reports the container's scroll position and size (see the module doc). `left` is the distance
/// from the inline start (`scrollLeft` is negative in RTL).
const SCROLL_BRIDGE_JS: &str = r#"
    const el = document.getElementById(await dioxus.recv());
    if (!el) return;
    let timer = null;
    let last = "";
    const publish = (isScrolling) => {
        const width = Math.min(el.clientWidth, window.innerWidth) || 800;
        const height = Math.min(el.clientHeight, window.innerHeight) || 600;
        const top = Math.round(el.scrollTop);
        const left = Math.round(Math.abs(el.scrollLeft));
        const key = top + " " + left + " " + width + " " + height + " " + isScrolling;
        if (key === last) return;
        last = key;
        dioxus.send({ top, left, width, height, isScrolling });
    };
    listen(el, "scroll", () => {
        if (timer !== null) clearTimeout(timer);
        // Sent at once (no rAF batching), so Rust sees it before the next render; Rust turns
        // most reports into no render at all (`virtualizer.rs`).
        publish(true);
        // The canvas size stays frozen until the scroll has been quiet this long. Firefox in CI
        // can take long enough between scroll events that a shorter timeout unfroze it mid-scroll.
        timer = setTimeout(() => {
            timer = null;
            publish(false);
        }, 600);
    }, { passive: true });
    // The visible height is capped at the window's, so a window resize can change it without
    // resizing the container.
    listen(window, "resize", () => publish(false), { passive: true });
    const observer = new ResizeObserver(() => publish(false));
    observer.observe(el);
    cleanup(() => {
        observer.disconnect();
        if (timer !== null) clearTimeout(timer);
    });
    publish(false);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    /// The bridge registers its listeners through `listen` (removed with the scope), never with
    /// a raw `addEventListener`, keeps them passive, and undoes the observer and the timer.
    #[test]
    fn the_scroll_bridge_uses_listen_and_is_passive() {
        assert!(SCROLL_BRIDGE_JS.contains(r#"listen(el, "scroll""#));
        assert!(SCROLL_BRIDGE_JS.contains(r#"listen(window, "resize""#));
        assert_eq!(SCROLL_BRIDGE_JS.matches("{ passive: true }").count(), 2);
        assert!(!SCROLL_BRIDGE_JS.contains("addEventListener"));
        assert_eq!(SCROLL_BRIDGE_JS.matches("new ResizeObserver").count(), 1);
        assert!(SCROLL_BRIDGE_JS.contains("observer.disconnect()"));
        assert!(SCROLL_BRIDGE_JS.contains("clearTimeout(timer)"));
    }

    /// The script sends exactly the fields `ScrollReport` deserializes (`rename_all =
    /// "camelCase"`: `is_scrolling` is `isScrolling`).
    #[test]
    fn the_report_shape_matches_the_script() {
        assert!(SCROLL_BRIDGE_JS.contains("dioxus.send({ top, left, width, height, isScrolling })"));
    }

    #[test]
    fn corrections_saturate_at_both_ends() {
        assert_eq!(corrected(100, 25), 125);
        assert_eq!(corrected(100, -25), 75);
        assert_eq!(corrected(10, -25), 0);
        assert_eq!(corrected(u32::MAX - 1, 5), u32::MAX);
    }

    #[test]
    fn measured_sizes_round_up_and_never_collapse_to_zero() {
        assert_eq!(measured_px(47.4), 48);
        assert_eq!(measured_px(66.1875), 67);
        assert_eq!(measured_px(47.0), 47);
        assert_eq!(measured_px(47.004), 47);
        assert_eq!(measured_px(0.0), 1);
        assert_eq!(measured_px(-3.0), 1);
        assert_eq!(measured_px(f64::NAN), 1);
        assert_eq!(measured_px(1e12), u32::MAX);
    }

    /// The bridge drives both engines through one trait: once a render has drawn the window,
    /// a report that leaves it in place asks for nothing and one that moves it asks for a render.
    #[test]
    fn both_engines_drive_through_one_trait() {
        fn asks<E: Windowing>(engine: &mut E) -> bool {
            engine.needs_render()
        }
        let mut list = Engine::new();
        list.sync(&super::super::SizeEstimates::evaluate(1_000, Some(&|_| 40)));
        let _ = list.scroll(0, 400, false);
        assert!(asks(&mut list), "the first known viewport needs a render");
        let _ = list.plan(8);
        assert!(!asks(&mut list), "drawn, and nothing moved");
        let _ = list.scroll(20_000, 400, false);
        assert!(asks(&mut list));

        let report = |top, left| ScrollReport {
            top,
            left,
            width: 800,
            height: 600,
            is_scrolling: false,
        };
        let mut grid = GridEngine::new();
        let rows = super::super::SizeEstimates::evaluate(1_000, Some(&|_| 36));
        let columns = super::super::SizeEstimates::evaluate(50, Some(&|_| 120));
        grid.sync(&rows, &columns, 0, 0, 0);
        grid.scroll(report(0, 0));
        assert!(asks(&mut grid));
        let _ = grid.plan(6, 4, None);
        assert!(!asks(&mut grid));
        grid.scroll(report(0, 4_000));
        assert!(
            asks(&mut grid),
            "a horizontal scroll moves the column window"
        );
    }
}
