//! Per-view scroll memory for the email client.
//!
//! A *view* is one scroll container showing one thing: the message list for a folder, tab and
//! filter, or one message's thread. Its id (`EmailClientState::list_view_id` / `read_view_id`)
//! is written on the container as `data-scroll-view`, in the same render that swaps its
//! content, so the DOM itself says which view its scroll offset belongs to.
//!
//! One rule for every view, by construction (instead of each switch resetting or forgetting
//! its own scroll): the offset is saved under the view id while the user scrolls, and whenever
//! the view id (or whether the pane is shown) changes, the container is set to the saved offset
//! of the view it now holds -- `0` for a view that was never visited. Saving only starts once
//! the container has been restored for its current view (`data-scroll-restored`), so the browser
//! clamping the offset when the content swaps can never overwrite a saved position.
//!
//! The offset belongs to the container, not the window: the page itself never scrolls here.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use dioxus::document;
use dioxus::prelude::*;
use dioxus_primitives::js_listener::{use_js_listeners, JsListeners};

/// Saved scroll offsets by view id. Plain shared storage, not a signal: a save per scroll frame
/// must not re-render anything. Held in [`EmailClientState`](super::state::EmailClientState) so
/// it lives exactly as long as the email client.
#[derive(Clone, Default)]
pub(super) struct ScrollMemory(Rc<RefCell<BTreeMap<String, f64>>>);

impl PartialEq for ScrollMemory {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl ScrollMemory {
    pub fn save(&self, view: String, offset: f64) {
        self.0.borrow_mut().insert(view, offset);
    }

    /// The saved offset of `view`; `0` (the top) for a view that was never visited.
    pub fn offset(&self, view: &str) -> f64 {
        self.0.borrow().get(view).copied().unwrap_or(0.0)
    }
}

/// Keeps the scroll offset of the container matching `selector` per view.
///
/// `view` returns the id of the view the container currently shows (it must be the same value
/// the container carries as `data-scroll-view`). `shown` is read only to re-run the restore
/// when the pane becomes visible again: a pane hidden with `display: none` loses its offset, so
/// coming back to the same view must put it back.
pub(super) fn use_scroll_memory(
    memory: ScrollMemory,
    selector: &'static str,
    view: impl Fn() -> String + 'static,
    shown: impl Fn() -> bool + 'static,
) {
    let saving = memory.clone();
    use_js_listeners(move || {
        let saving = saving.clone();
        Some(
            JsListeners::install(
                r#"
                const selector = await dioxus.recv();
                let frame = 0;
                let pending = null;
                const flush = () => {
                    frame = 0;
                    if (pending) { dioxus.send(pending); pending = null; }
                };
                listen(document, 'scroll', (event) => {
                    const el = event.target;
                    if (!(el instanceof Element) || !el.matches(selector)) return;
                    const view = el.dataset.scrollView;
                    if (!view || el.dataset.scrollRestored !== view) return;
                    pending = [view, el.scrollTop];
                    if (!frame) frame = requestAnimationFrame(flush);
                }, true);
                cleanup(() => { if (frame) cancelAnimationFrame(frame); });
                "#,
            )
            .arg(selector)
            .on_message(move |(view, offset): (String, f64)| saving.save(view, offset)),
        )
    });

    use_effect(move || {
        let view = view();
        let _ = shown();
        let offset = memory.offset(&view);
        let mut eval = document::eval(
            r#"
            const selector = await dioxus.recv();
            const view = await dioxus.recv();
            const offset = await dioxus.recv();
            const apply = () => {
                const el = document.querySelector(selector);
                if (!el || el.dataset.scrollView !== view) return;
                if (Math.abs(el.scrollTop - offset) > 0.5) el.scrollTop = offset;
                el.dataset.scrollRestored = view;
            };
            apply();
            requestAnimationFrame(apply);
            dioxus.send(true);
            "#,
        );
        let _ = eval.send(selector);
        let _ = eval.send(view);
        let _ = eval.send(offset);
        spawn(async move {
            let _ = eval.recv::<bool>().await;
        });
    });
}
