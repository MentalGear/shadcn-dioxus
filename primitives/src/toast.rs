//! Defines the [`Toast`] component and its sub-components, which provide a notification system for displaying temporary messages to users.

use crate::{
    merge_attributes,
    portal::{use_portal, PortalIn, PortalOut},
    use_unique_id,
};
use dioxus::prelude::*;
use dioxus_attributes::attributes;
use std::collections::VecDeque;
use std::time::Duration;

// `document`/`Cell` are only needed for the web-arm's engine-injected
// baseline stylesheet (`ensure_toast_base_styles`, dev-docs/backlog.md row 44)
// -- same shape as `top_layer.rs`'s `ensure_anchor_positioning_styles`,
// which this mirrors.
#[cfg(feature = "web")]
use dioxus::document;
#[cfg(feature = "web")]
use std::cell::Cell;

/// Toast types for different visual styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    /// A success toast
    Success,
    /// An error toast
    Error,
    /// A warning toast
    Warning,
    /// An info toast
    Info,
    /// A toast for work in progress: it shows a spinner and is never auto-dismissed while it
    /// is loading. Move it on with [`Toasts::update`] (or let [`Toasts::promise`] do it).
    Loading,
}

impl ToastType {
    fn as_str(&self) -> &'static str {
        match self {
            ToastType::Success => "success",
            ToastType::Error => "error",
            ToastType::Warning => "warning",
            ToastType::Info => "info",
            ToastType::Loading => "loading",
        }
    }

    /// The ARIA live-region role of a toast of this type: an error interrupts (`alert`), every
    /// other type is polite (`status`). The ONE place the mapping lives.
    fn live_role(&self) -> &'static str {
        match self {
            ToastType::Error => "alert",
            _ => "status",
        }
    }
}

/// The handle of one toast, returned by [`Toasts::show`] and the typed shortcuts.
///
/// Correct by construction: the id is an opaque newtype (no public constructor, no arithmetic,
/// no conversion from or to an integer) allocated from one process-wide monotonic counter, so
/// it cannot be confused with the id of any other toast (not even one of a second
/// [`ToastProvider`]) and an id is never reused. A closed toast's id stays dead for good:
/// [`Toasts::update`] and [`Toasts::close`] on it are no-ops, never a hit on a newer toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ToastId(u64);

impl ToastId {
    fn next() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ToastId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Lets an event handler end in `toast.info(..)` without a semicolon, as it could before `show`
/// returned a handle: a `ToastId` is a plain value, so "returning it from a handler" does nothing.
impl crate::dioxus_core::SpawnIfAsync<()> for ToastId {
    fn spawn(self) {}
}

impl std::fmt::Display for ToastId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An action button on a toast (shadcn's `actionProps`, e.g. "Undo"). Activating it (click,
/// Enter or Space on the focused button) runs `on_click` and then dismisses the toast.
#[derive(Clone, PartialEq)]
pub struct ToastAction {
    /// The button's label.
    pub label: String,
    /// Called when the button is activated.
    pub on_click: Callback<()>,
}

impl ToastAction {
    /// An action with a label and a callback.
    pub fn new(label: impl ToString, on_click: impl FnMut(()) + 'static) -> Self {
        Self {
            label: label.to_string(),
            on_click: Callback::new(on_click),
        }
    }
}

impl std::fmt::Debug for ToastAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToastAction")
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}

// A single toast item
#[derive(Debug, Clone, PartialEq)]
struct ToastRecord {
    id: ToastId,
    /// Bumped by every [`Toasts::update`]: restarts the auto-dismiss countdown.
    revision: u32,
    title: String,
    description: Option<String>,
    toast_type: ToastType,
    /// The caller's own duration; the provider default fills in at render.
    duration: Option<Duration>,
    permanent: bool,
    action: Option<ToastAction>,
    /// Set by [`close_record`]: the toast is on its way out (its exit animation plays) and is
    /// removed [`TOAST_EXIT`] later. A closing toast no longer counts toward the pile.
    closing: bool,
}

impl ToastRecord {
    /// How long the toast stays, or `None` for "until closed". A loading toast is never
    /// auto-dismissed (derived here, from the type, so an update from loading to success
    /// starts the countdown by itself), and neither is a permanent one.
    fn effective_duration(&self, default: Option<Duration>) -> Option<Duration> {
        if self.permanent || self.toast_type == ToastType::Loading {
            None
        } else {
            self.duration.or(default)
        }
    }
}

// Type alias for the add callback
type AddToastCallback = Callback<ToastRecord>;

// Context for managing toasts
#[derive(Clone)]
struct ToastCtx {
    toasts: Signal<VecDeque<ToastRecord>>,
    add_toast: AddToastCallback,
    remove_toast: Callback<ToastId>,
    focus_region: Callback,
    /// `true` while the pointer is over the toaster or focus is inside it: every countdown waits.
    paused: ReadSignal<bool>,
    /// How many toasts of the pile are shown; the rest wait, hidden, for a slot to free up.
    visible_toasts: ReadSignal<usize>,
}

/// How long a closed toast stays mounted so its exit animation can play (Sonner's
/// `TIME_BEFORE_UNMOUNT`). It is also written on the toast as `--toast-exit-ms`, which the
/// stylesheet's exit transition reads, so the Rust constant is the one source of the duration.
const TOAST_EXIT: Duration = Duration::from_millis(200);

/// Extra time a closed toast stays mounted after [`TOAST_EXIT`]. The exit transitions start on the
/// first frame AFTER the `closing` render, not at the moment the timer starts, so unmounting at
/// exactly `TOAST_EXIT` could cut the last few percent of the pile's gap-closing off and show them
/// as a one-frame jump. The slack makes "every exit transition has finished" true by construction.
const TOAST_UNMOUNT_SLACK: Duration = Duration::from_millis(60);

/// Closes the toast `id` with its exit animation: marks it `closing` (so it gets `data-removed`,
/// leaves the pile at once and is inert) and removes it after [`TOAST_EXIT`]. Closing a toast
/// that is closing or gone does nothing. EVERY dismissal goes through here (close button, action,
/// countdown, [`Toasts::close`], a swipe): the toast's list item takes its place out of the layout
/// at once (`data-removed` on the `li`), so the toasts that stay close the gap while it fades and no
/// one jumps when it unmounts.
fn close_record(mut toasts: Signal<VecDeque<ToastRecord>>, id: ToastId) {
    {
        let Ok(mut list) = toasts.try_write() else {
            return;
        };
        let Some(toast) = list.iter_mut().find(|t| t.id == id && !t.closing) else {
            return;
        };
        toast.closing = true;
    }
    // interval-ok: one bounded wait, dropped with the app
    crate::dioxus_core::spawn_forever(async move {
        crate::sleep::sleep(TOAST_EXIT + TOAST_UNMOUNT_SLACK).await;
        remove_record(toasts, id);
    });
}

/// Removes the record `id`, if it still exists. `try_write`: a promise that settles after its
/// provider is gone must not panic.
fn remove_record(mut toasts: Signal<VecDeque<ToastRecord>>, id: ToastId) {
    let Ok(mut toasts_vec) = toasts.try_write() else {
        return;
    };
    if let Some(pos) = toasts_vec.iter().position(|t| t.id == id) {
        toasts_vec.remove(pos);
    }
}

// Toast provider props
/// The props for the [`ToastProvider`] component
#[derive(Props, Clone, PartialEq)]
pub struct ToastProviderProps {
    /// The default duration for non-permanent toasts. Defaults to 5 seconds
    #[props(default = ReadSignal::new(Signal::new(Some(Duration::from_secs(5)))))]
    pub default_duration: ReadSignal<Option<Duration>>,

    /// The maximum number of toasts to display at once. Defaults to 10.
    #[props(default = ReadSignal::new(Signal::new(10)))]
    pub max_toasts: ReadSignal<usize>,

    /// How many toasts the pile shows (Sonner's `visibleToasts`). Defaults to 3. A toast past
    /// the cap stays mounted but hidden (`data-visible="false"`), expanded or not, and shows up
    /// when a slot frees.
    #[props(default = ReadSignal::new(Signal::new(3)))]
    pub visible_toasts: ReadSignal<usize>,

    /// The callback to render a toast. Defaults to rendering the [`Toast`] component.
    #[props(default = Callback::new(|props: ToastPropsWithOwner| rsx! { Toast { ..props } }))]
    pub render_toast: Callback<ToastPropsWithOwner, Element>,

    /// Additional attributes to apply to the toast container element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the toast provider component.
    pub children: Element,
}

/// The props for the [`ToastList`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ToastListProps {
    /// Additional attributes to apply to the toast list element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the toast list element.
    pub children: Element,
}

/// The props for the [`ToastListItem`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ToastListItemProps {
    /// Additional attributes to apply to the toast list item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the toast list item element.
    pub children: Element,
}

/// # ToastProvider
///
/// The provider component manages rendering any toasts sent by child components. This component should wrap all components that need access to the [`use_toast`] hook.
///
/// It provides a global `f6` shortcut to focus the toast region, allowing users to quickly access the most recent toast notifications.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::toast::{ToastOptions, ToastProvider, use_toast};
/// use std::time::Duration;
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ToastProvider { ToastButton {} }
///     }
/// }
///
/// #[component]
/// fn ToastButton() -> Element {
///     let toast_api = use_toast();
///
///     rsx! {
///         button {
///             onclick: move |_| {
///                 toast_api
///                     .info(
///                         "Custom Toast".to_string(),
///                         ToastOptions::new()
///                             .description("Some info you need")
///                             .duration(Duration::from_secs(60))
///                             .permanent(false),
///                     );
///             },
///             "Info (60s)"
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`ToastProvider`] component renders toasts with the following css variables you can use to control styling:
/// - `--data-toast-count`: The number of toasts currently displayed.
#[component]
pub fn ToastProvider(props: ToastProviderProps) -> Element {
    let mut toasts = use_signal(VecDeque::new);
    let portal = use_portal();

    // Remove toast callback
    let remove_toast = use_callback(move |id: ToastId| close_record(toasts, id));

    // Add toast callback
    let add_toast = use_callback(move |toast: ToastRecord| {
        // Add the toast directly to the queue
        // This is safe because we're in an event handler, not during render
        let mut toasts_vec = toasts.write();
        toasts_vec.push_back(toast);

        // Limit the number of toasts, but prioritize keeping permanent toasts
        let max = (props.max_toasts)();
        while toasts_vec.len() > max {
            // Try to find a non-permanent toast to remove first
            if let Some(pos) = toasts_vec.iter().position(|t| !t.permanent) {
                toasts_vec.remove(pos);
            } else {
                // If all toasts are permanent, remove the oldest one
                toasts_vec.pop_front();
            }
        }

        // We'll handle auto-dismissal in the Toast component
    });

    // Create a stable list of toasts for rendering outside of RSX
    let toast_list = use_memo(move || {
        let toasts_vec = toasts.read();
        toasts_vec.iter().cloned().collect::<Vec<_>>()
    });
    let length = toast_list.len();

    let mut region_ref: Signal<Option<std::rc::Rc<MountedData>>> = use_signal(|| None);

    let focus_region = use_callback(move |_| {
        let Some(region_ref) = region_ref() else {
            return;
        };
        spawn(async move {
            // focus-ok: the hotkey moves focus to the toast region on purpose
            _ = region_ref.set_focus(true).await;
        });
    });

    // A stable id for the region: `ToastRegionRendered`'s popover wiring, the F6 registry and the
    // pause listener all address the element by it.
    let region_id = use_unique_id();

    // F6: this provider only REGISTERS its region (and how many toasts it holds) with the one
    // shared handler (`TOAST_F6_JS`); it never listens itself, so N providers on a page are
    // still one listener and F6 can pick the right region. Declared before the effect that
    // reports, so the shared registry exists by the time the first report arrives.
    crate::js_listener::use_js_listeners(move || {
        let id = region_id.cloned();
        Some(crate::js_listener::JsListeners::install(TOAST_F6_JS).arg(id))
    });
    use_effect(move || {
        let newest = toast_list.read().last().map_or(0, |t| t.id.0);
        let length = toast_list.read().len();
        let _ = document::eval(&format!(
            "window.__dxToastF6&&window.__dxToastF6.set({:?},{length},{newest})",
            region_id.cloned()
        ));
    });

    // Pause the countdowns while the pointer is over the toaster or focus is inside it.
    let mut paused = use_signal(|| false);
    crate::js_listener::use_js_listeners(move || {
        region_ref()?;
        let id = region_id.cloned();
        Some(
            crate::js_listener::JsListeners::install(TOAST_PAUSE_JS)
                .arg(id)
                .on_message(move |p: bool| paused.set(p)),
        )
    });

    // Measure every toast's content and publish the heights the pile is sized from (`TOAST_SIZE_JS`).
    crate::js_listener::use_js_listeners(move || {
        region_ref()?;
        let id = region_id.cloned();
        Some(crate::js_listener::JsListeners::install(TOAST_SIZE_JS).arg(id))
    });

    // Provide the context
    let ctx = use_context_provider(|| ToastCtx {
        toasts,
        add_toast,
        remove_toast,
        focus_region,
        paused: paused.into(),
        visible_toasts: props.visible_toasts,
    });

    rsx! {
        // Render children
        {props.children}

        // Render toast container using portal
        PortalIn { portal,
            ToastRegionRendered {
                id: region_id.cloned(),
                length,
                attributes: props.attributes,
                onmounted: move |e: MountedEvent| {
                    region_ref.set(Some(e.data()));
                },

                ToastList {
                    // Render all toasts
                    for (index, toast) in toast_list.read().iter().rev().scan(0usize, |live, toast| {
                        // The pile position counts the toasts that are not closing: a closing
                        // toast leaves the pile at once (the others restack under its exit).
                        let index = *live;
                        if !toast.closing {
                            *live += 1;
                        }
                        Some((index, toast))
                    }) {
                        ToastListItem {
                            key: "{toast.id}",
                            // The slot leaves the layout the moment the toast starts leaving: the
                            // stylesheet shrinks its footprint over `--toast-exit-ms`, so the
                            // toasts that stay move smoothly instead of jumping at the unmount.
                            class: "dx-toast-slot",
                            "data-removed": toast.closing.then_some("true"),
                            style: "--toast-exit-ms: {TOAST_EXIT.as_millis()}ms",
                            {
                                props.render_toast.call(ToastProps::builder().id(toast.id)
                                    .index(index)
                                    .revision(toast.revision)
                                    .title(toast.title.clone())
                                    .description(toast.description.clone())
                                    .toast_type(toast.toast_type)
                                    .permanent(toast.permanent)
                                    .action(toast.action.clone())
                                    .on_close({
                                        let toast_id = toast.id;
                                        let remove_toast = ctx.remove_toast;
                                        move |_| {
                                            remove_toast.call(toast_id);
                                        }
                                    })
                                    // A loading or permanent toast gets no duration
                                    .duration(toast.effective_duration((props.default_duration)()))
                                    .attributes(vec![])
                                    .build()
                                )
                            }
                        }
                    }
                }
            }
        }

        // Portal output at the end of the document
        PortalOut { portal }
    }
}

/// The ONE F6 ("jump to notifications") handler of the page, shared by every provider.
///
/// Each provider runs this script through `use_js_listeners` (so it ends with the provider). The
/// first one creates `window.__dxToastF6` and the single `keydown` listener behind it; every
/// provider takes a reference, and the last one to leave removes the listener again (the
/// registry is reference-counted, like `activity.rs`'s, so a page without a toaster holds no
/// listener at all and `listener-inventory.spec.ts` stays at its baseline). Providers report
/// `(region id, toast count, newest toast id)` through `window.__dxToastF6.set`. F6 moves focus to
/// the region of the toast that arrived last (`ToastId`s come from one process-wide counter, so
/// "newest" needs no clock); pressed again from inside a region it goes on to the next one,
/// most recent first, and wraps. Regions without toasts, or no longer in the document, are never
/// candidates, and with no candidate F6 is left to the browser.
const TOAST_F6_JS: &str = r#"
const id = await dioxus.recv();
let R = window.__dxToastF6;
if (!R) {
  const regs = new Map();
  const onKey = (e) => {
    if (e.key !== 'F6') return;
    const live = [];
    regs.forEach((newest, rid) => {
      const el = document.getElementById(rid);
      if (el) live.push({ el, newest });
    });
    if (!live.length) return;
    live.sort((a, b) => b.newest - a.newest);
    let cur = -1;
    live.forEach((r, i) => { if (r.el.contains(document.activeElement)) cur = i; });
    e.preventDefault();
    // focus-ok: the hotkey moves focus to the toast region on purpose
    live[(cur + 1) % live.length].el.focus();
  };
  // listener-ok: the ONE F6 handler, reference-counted below and removed with the last provider
  document.addEventListener('keydown', onKey);
  R = window.__dxToastF6 = {
    refs: 0,
    regs,
    set(rid, n, newest) { if (n > 0) regs.set(rid, newest); else regs.delete(rid); },
    off() { document.removeEventListener('keydown', onKey); },
  };
}
R.refs++;
cleanup(() => {
  R.regs.delete(id);
  if (--R.refs === 0) {
    R.off();
    if (window.__dxToastF6 === R) delete window.__dxToastF6;
  }
});
"#;

/// Reports whether the pointer is over the toaster (a mouse or pen; a touch tap is not a hover)
/// or focus is inside it, as `true`/`false` on every change. The same condition the stylesheet
/// expands the pile on (`:hover`, `:focus-within`), so the pile and the countdowns agree.
const TOAST_PAUSE_JS: &str = r#"
const id = await dioxus.recv();
const el = document.getElementById(id);
if (!el) return;
let hover = false, focus = false, last = false;
const report = () => {
  const p = hover || focus;
  if (p !== last) { last = p; dioxus.send(p); }
};
listen(el, 'pointerenter', (e) => { if (e.pointerType !== 'touch') { hover = true; report(); } });
listen(el, 'pointerleave', () => { hover = false; report(); });
listen(el, 'focusin', () => { focus = true; report(); });
listen(el, 'focusout', (e) => { focus = !!e.relatedTarget && el.contains(e.relatedTarget); report(); });
"#;

/// The ONE writer of the pile's heights. A toast is as tall as its content plus its own padding, so
/// the stylesheet needs two numbers it cannot read from CSS: how tall each toast is, and how tall the
/// FRONT toast is (a collapsed pile shows every toast behind it at that height, as Sonner's
/// `--front-toast-height` does).
///
/// A skin marks the box that holds a toast's content with `data-toast-body` (a direct child of the
/// toast, no margin of its own; the toast's padding and border go around it). A `ResizeObserver` watches
/// every such box in the region's list and writes, on the toast's list item (`li`, which Rust never
/// restyles, so a re-render cannot drop it), `--toast-height`: the body's height plus the toast's computed
/// padding and border. The front toast, the first list item that is not closing (`data-removed`), also
/// publishes its height as `--front-toast-height` on the list. The first measurement of a toast also sets
/// `data-sized` on its `li` (`pending`, `true` one frame later), so a skin can keep the unmeasured to
/// measured switch from animating. A `MutationObserver` on the list follows toasts coming, going and
/// starting to leave, and measures a new toast as soon as it is inserted, so no frame lays it out
/// unmeasured. Both observers end with the provider (`cleanup`). A skin
/// without `data-toast-body` is never measured: its toasts keep their auto height and do not overlap.
const TOAST_SIZE_JS: &str = r#"
const id = await dioxus.recv();
const region = document.getElementById(id);
const list = region && region.querySelector('ol');
if (!list) return;
const heights = new WeakMap();
const seen = new Set();
const FRONT = '--front-toast-height';
// The toast's own height: its content box plus the padding and border around it.
const heightOf = (body, size) => {
  const cs = getComputedStyle(body.parentElement);
  const chrome = parseFloat(cs.paddingTop) + parseFloat(cs.paddingBottom)
    + parseFloat(cs.borderTopWidth) + parseFloat(cs.borderBottomWidth);
  const box = size && size.borderBoxSize && size.borderBoxSize[0];
  return (box ? box.blockSize : body.offsetHeight) + (chrome || 0);
};
const publishFront = () => {
  const front = Array.from(list.children).find((li) => li.dataset.removed !== 'true');
  const h = front && heights.get(front);
  if (h) list.style.setProperty(FRONT, h + 'px');
  else list.style.removeProperty(FRONT);
};
const record = (body, size) => {
  const li = body.closest('li');
  if (!li) return;
  const h = heightOf(body, size);
  heights.set(li, h);
  li.style.setProperty('--toast-height', h + 'px');
  // The first measurement must not play as a glide from the unmeasured layout: `data-sized` turns
  // true a frame later, and the stylesheet only transitions a toast's height and margin from then on.
  if (!li.dataset.sized) {
    li.dataset.sized = 'pending';
    requestAnimationFrame(() => { li.dataset.sized = 'true'; });
  }
};
const sizes = new ResizeObserver((entries) => {
  for (const entry of entries) record(entry.target, entry);
  publishFront();
});
const sync = () => {
  const now = new Set(list.querySelectorAll('[data-toast-body]'));
  for (const body of seen) if (!now.has(body)) { sizes.unobserve(body); seen.delete(body); }
  // A new toast is measured at once, in the mutation callback that sees it inserted: the observer's
  // first callback comes after the next frame's rAF callbacks, so a frame would lay the pile out
  // without the toast's height and the slot it enters through would open by the gap alone.
  for (const body of now) if (!seen.has(body)) { seen.add(body); record(body); sizes.observe(body); }
  publishFront();
};
const changes = new MutationObserver(sync);
changes.observe(list, { childList: true, subtree: true, attributes: true, attributeFilter: ['data-removed'] });
cleanup(() => { changes.disconnect(); sizes.disconnect(); list.style.removeProperty(FRONT); });
sync();
"#;

#[cfg(feature = "web")]
thread_local! {
    /// Whether [`ensure_toast_base_styles`]'s `document::eval` has already
    /// been scheduled once in this WASM instance -- same idempotency guard
    /// shape as `top_layer.rs`'s `ANCHOR_POSITIONING_STYLES_INSTALLED`
    /// (itself modeled on `scroll_lock.rs`'s
    /// `SCROLLBAR_GUTTER_BASELINE_INSTALLED`).
    static TOAST_BASE_STYLES_INSTALLED: Cell<bool> = const { Cell::new(false) };
}

/// Installs a minimal, always-applied fallback stylesheet for the raw
/// primitive's own markup, at most once per WASM instance -- dev-docs/backlog.md
/// row 44, construction (2).
///
/// ## The bug this closes
///
/// `ToastRegionRendered`'s `div[popover]` and [`Toast`]'s
/// `div[role="alertdialog"]` carry no layout CSS of their own; every real
/// box they get comes from whatever a *consumer's* stylesheet supplies
/// (`preview/src/components/toast/style.css`'s `.dx-toast-container[popover]`/
/// `.dx-toast`, for the themed wrapper -- see `dev-docs/preview-composition.md`).
/// `preview/src/components/top_layer/component.rs`'s oracle fixture
/// deliberately composes this raw primitive with no such class at all (its
/// own header doc: calibrated against native `popover`/top-layer mechanics,
/// not a themed look), so before this function existed its toast had
/// whatever box the *engine's own* UA popover-stylesheet defaults
/// (`[popover] { inset: 0; margin: auto; border: solid; padding: 0.25em;
/// overflow: auto; background-color: Canvas; }`) plus `fit-content` sizing
/// happened to produce -- per dev-docs/backlog.md row 44's own root-cause
/// analysis, this is a real gap between engines: Chromium's default gives
/// it a small (~105x40px) on-screen box (confirmed there by execution
/// against a real build), while a 2026-09-03 iOS 18 device report of the
/// fixture's "Add toast" appearing dead is the unverified-but-plausible
/// WebKit instance of the same gap -- this crate has no WebKit build to
/// confirm that half against directly. Rather than tie the
/// fixture's deliberately-raw composition to the theme (which would also
/// mean removing `scripts/check-preview-composition.sh`'s stated exemption
/// for it, a bigger and unrelated call), this gives the *primitive itself*
/// a minimal, engine-independent layout baseline -- the same idea
/// `top_layer.rs`'s `ensure_anchor_positioning_styles` already applies for
/// anchor positioning, just for toast's own box instead.
///
/// ## Zero specificity, deliberately
///
/// Every rule below is wrapped in `:where(...)`, exactly like
/// `top_layer.rs`'s own `[popover]` reset -- see that function's doc for
/// why: it lets a themed wrapper's real, higher-specificity CSS
/// (`.dx-toast-container[popover]`, `.dx-toast`, both plain classes with
/// nonzero specificity) win outright for every property it declares, so
/// this baseline only ever fills a gap a themed page's own stylesheet
/// leaves, never fights it.
///
/// Call this once per `ToastRegionRendered` mount (it is idempotent, so
/// calling it more than once is harmless) -- mirrors
/// `use_anchor_position_fallback`'s own `use_effect(ensure_anchor_
/// positioning_styles)` call for the identical reason.
#[cfg(feature = "web")]
fn ensure_toast_base_styles() {
    if TOAST_BASE_STYLES_INSTALLED.with(|installed| installed.replace(true)) {
        return;
    }
    let eval = document::eval(TOAST_BASE_STYLES_INJECT_JS);
    let _ = eval;
}

/// The idempotent (JS-side `getElementById` guarded, so safe to run more
/// than once -- same double-guard shape as `top_layer.rs`'s
/// `anchor_positioning_inject_js`) style-tag-injection script
/// [`ensure_toast_base_styles`] dispatches.
///
/// `dx-toast-base-region`/`dx-toast-base-item` are plain, hand-written
/// marker classes (never `#[css_module]`-routed -- this crate has no
/// build-time CSS pipeline of its own, only `preview/`'s does), applied
/// unconditionally by `ToastRegionRendered`/[`Toast`] alongside whatever
/// class a themed wrapper also merges in via `merge_attributes`'s
/// concatenating `class` rule.
#[cfg(feature = "web")]
const TOAST_BASE_STYLES_INJECT_JS: &str = r#"
if (!document.getElementById('dx-toast-base-styles')) {
    const style = document.createElement('style');
    style.id = 'dx-toast-base-styles';
    style.textContent = `
      :where(.dx-toast-base-region[popover]) {
        position: fixed;
        inset: auto 1rem 1rem auto;
        margin: 0;
        overflow: visible;
        border: none;
        padding: 0;
        background: none;
        color: var(--dx-foreground, inherit);
      }
      :where(.dx-toast-base-region ol) {
        display: flex;
        flex-direction: column-reverse;
        margin: 0;
        padding: 0;
        gap: 0.5rem;
        list-style: none;
      }
      :where(.dx-toast-base-region li) {
        display: flex;
      }
      :where(.dx-toast-base-item) {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.5rem;
        box-sizing: border-box;
        min-width: 12rem;
        min-height: 2.5rem;
        padding: 0.75rem 1rem;
        border: 1px solid CanvasText;
        border-radius: 0.25rem;
        background: Canvas;
        color: CanvasText;
      }
    `;
    document.head.appendChild(style);
}
"#;

/// Web arm (Migration A slice 3/3, final): promote the toast region to the
/// top layer via `popover="manual"` so toasts are never clipped by an
/// `overflow: hidden` ancestor and always paint above everything else on
/// the page (WHATWG HTML's top layer), the same fix already shipped for
/// every other non-modal overlay in this crate (dev-docs/plan.md Phase 4.4,
/// Migration A slices 1-3).
///
/// `manual`, not `auto` -- deliberately, and unlike every other migrated
/// component in this crate: this region has no open/close lifecycle at
/// all. It is mounted once, for the lifetime of the `ToastProvider`, and
/// its own visibility never depends on whether it currently holds any
/// toasts (`ToastList`'s `for` loop above is empty when there are none --
/// the region itself stays mounted regardless, so `use_global_keydown_
/// listener("F6", ..)` can always find and focus it). There is nothing
/// here for `auto`'s light dismiss to ever legitimately fire on (Escape or
/// an outside click closing the whole notification region would be
/// actively wrong -- toasts dismiss individually, via their own close
/// button or timeout, never as a group), so this calls
/// [`crate::top_layer::use_popover_shown_while_mounted`] with a no-op
/// native-close callback: nothing should ever native-close a `manual`
/// popover in the first place (WHATWG HTML never light-dismisses one), so
/// this is a defensive no-op, matching `ContextMenuContentRendered`'s
/// identical reasoning for its own `manual` round trip
/// (`context_menu.rs`).
///
/// No anchoring: unlike every other migrated overlay, toasts are not
/// positioned relative to a trigger at all -- `.dx-toast-container` in the
/// consuming page's stylesheet fixes them to a viewport corner
/// (`position: fixed; right: 20px; bottom: 20px`) via plain CSS, with no
/// `anchor-name`/`position-anchor` involved, so neither
/// `crate::top_layer::anchor_name_style` nor `use_anchor_position_fallback`
/// is called here, and this component carries no `dx-anchor-*` marker
/// class (that class only exists to opt an element into the shared,
/// engine-injected `@supports (anchor-name: --a)` block, which has nothing
/// to contribute here).
///
/// ## The UA popover reset this needed (found by execution)
///
/// The WHATWG popover UA stylesheet's `[popover] { inset: 0; margin: auto;
/// border: solid; padding: 0.25em; overflow: auto; background-color:
/// Canvas; color: CanvasText; }` fights this region's own positioning the
/// instant the attribute is added: author CSS already sets `right`/`bottom`
/// (not `top`/`left`), so the UA's `inset: 0` still supplies `top: 0; left:
/// 0` (author origin only overrides the *properties it actually
/// declares*), and `margin: auto` (which the pre-popover initial value of
/// `0` never triggered) combines with those to invoke the CSS
/// auto-margin/over-constrained sizing algorithm -- confirmed by execution,
/// this stretched the region across nearly the entire viewport instead of
/// shrink-wrapping the toast list in its corner. `border`/`padding`/
/// `background-color`/`color` also all apply for the same reason (the
/// region declares none of them itself, only its child `.dx-toast`s do),
/// painting a visible boxed panel behind the toasts that was never there
/// before. The consuming page's own stylesheet
/// (`preview/src/components/toast/style.css`'s `.dx-toast-container[popover]`)
/// resets all of these back to their pre-popover values -- the same
/// `margin: 0; inset: auto;` idiom the shared anchor-positioning engine
/// uses for anchored overlays (`top_layer.rs`'s
/// `ensure_anchor_positioning_styles` doc), applied here directly in the
/// component's own stylesheet instead of that shared engine, since this
/// region needs the reset without ever wanting anchor positioning itself.
///
/// ## Attribute-override dedup (found by execution, 2026-09-01)
///
/// `attributes` used to be spread (`..attributes`) directly after this
/// element's own explicit `id`/`role`/`aria_label`/`tabindex`/`popover`/
/// `style`. On the client that's harmless -- Dioxus applies attributes to
/// the DOM sequentially, so a caller's later value in `..attributes` simply
/// wins -- but the SSR renderer serializes both the explicit attribute and
/// the caller's override as two separate `attr="..."` occurrences in the
/// same start tag, and WHATWG HTML's "duplicate attribute" parse error has
/// browsers keep the *first* one instead. Server and client then disagree
/// about which value won (`dev-docs/conformance-harness.md`'s hydration-parity
/// Rule 4) -- concretely, a caller's `aria_label` override (two
/// `ToastProvider`s on one page needing distinct accessible names, e.g.
/// `preview/src/components/top_layer/component.rs`) rendered as the
/// *primitive's* default `"{length} notifications"` in the served HTML
/// while the live DOM showed the caller's text, failing axe's
/// `landmark-unique` only on the prerendered/SSG lane. Every attribute
/// below except `id` is merged with caller-wins semantics via
/// `merge_attributes` instead, so exactly one value is ever emitted, on
/// both lanes. `id` is different: per this function's own doc above, it
/// must never be caller-overridable at all (a caller's `id` would strand
/// `showPopover()`'s `getElementById` lookup on a dead id), so instead of
/// relying on spread order to keep the real one live, any `id` a caller's
/// `attributes` may carry is dropped explicitly before merging -- non-
/// overridable by construction on both lanes, not just by accident of
/// spread order.
#[cfg(feature = "web")]
#[component]
fn ToastRegionRendered(
    id: String,
    length: usize,
    attributes: Vec<Attribute>,
    onmounted: EventHandler<MountedEvent>,
    children: Element,
) -> Element {
    // Always `true`: this region is mounted for the lifetime of
    // `ToastProvider` and never closes, so it only ever needs the initial
    // `showPopover()` -- see `use_popover_shown_while_mounted`'s doc,
    // "Bug 2," for why real `open`-driven content needs this to react on
    // every transition instead of a one-shot mount effect (not applicable
    // here: `manual` never lets native light dismiss hide this out from
    // under us the way `auto` can for `Select`/`Combobox`).
    //
    // `id` above is this function's own prop (`ToastProvider`'s internal,
    // `use_unique_id`-generated region id -- `ToastProviderProps` has no
    // caller-facing `id` field of its own), not anything read out of
    // `attributes` -- keep it that way: an `id` a caller passed through
    // `ToastProviderProps`'s `#[props(extends = GlobalAttributes)]`
    // catch-all would land in `attributes` and, spread via `..attributes`
    // below (after this `id: id.clone()`), silently win and override it,
    // stranding `showPopover()`'s `document.getElementById(id)` above on a
    // dead id that matches no element (confirmed by execution against this
    // exact mistake in an earlier version of the top-layer oracle fixture,
    // `preview/src/components/top_layer/component.rs` -- see its own
    // comment on this).

    // Installs `ensure_toast_base_styles`'s engine-injected fallback
    // stylesheet -- see its own doc for the bug this closes (dev-docs/backlog.md
    // row 44) and why zero relative-ordering risk exists here (unlike
    // `use_anchor_position_fallback`'s pairing with a same-tick measurement
    // eval, nothing below reads computed layout, so a separate `use_effect`
    // call is sufficient, no same-script prepending needed).
    use_effect(ensure_toast_base_styles);

    let always_open = use_signal(|| true);
    crate::top_layer::use_popover_shown_while_mounted(
        id.clone(),
        always_open,
        Callback::new(|_: bool| {}),
    );

    // See "Attribute-override dedup" above: drop any caller `id` (never
    // overridable) and merge everything else with caller-wins semantics so
    // neither lane ever emits a duplicate attribute.
    //
    // `style` needs its own pass first, though (dev-docs/backlog.md row 44,
    // found by execution): a caller's own plain `style` attribute (e.g. the
    // `top_layer` preview fixture's `ToastProvider { style: "position:
    // fixed; ..." }`) is the *same* `(name, namespace)` key --
    // `("style", None)` -- as this element's own `style: "--toast-count:
    // {length}"` literal below, so plain `merge_attributes` sees them as one
    // collision and keeps only the last-in-the-list value (the caller's),
    // silently dropping `--toast-count` -- the exact same duplicate-`style`
    // hazard `top_layer::anchored_content_attributes` exists to fix for the
    // anchored-content family (its own doc). `crate::fold_style_attributes`
    // pulls whatever `style` the caller supplied (plain literal or
    // shorthand props) out as one string first, so it can be folded
    // together with `--toast-count` into a single resulting `style`
    // instead of two competing ones -- `--toast-count` first so the
    // caller's own declarations, listed after, can still override any
    // property they want without ever being able to drop the counter
    // itself (same ordering rationale as `anchored_content_attributes`).
    let attributes: Vec<Attribute> = attributes.into_iter().filter(|a| a.name != "id").collect();
    let (caller_style, attributes) = crate::fold_style_attributes(attributes);
    let region_style = match caller_style {
        Some(caller_style) => format!("--toast-count: {length}; {caller_style}"),
        None => format!("--toast-count: {length}"),
    };
    let merged = merge_attributes(vec![
        attributes!(div {
            role: "region",
            aria_label: "{length} notifications",
            tabindex: "-1",
            popover: crate::top_layer::PopoverKind::Manual.as_str(),
            style: "{region_style}",
            // See `ensure_toast_base_styles`'s doc: a plain, always-present
            // marker class the engine-injected fallback stylesheet selects
            // on, merged (space-concatenated, `merge_attributes`'s own
            // `class` rule) with whatever class a themed wrapper also
            // passes through `attributes` -- never overridden by one, only
            // ever added alongside it.
            class: "dx-toast-base-region",
        }),
        attributes,
    ]);

    rsx! {
        div {
            id: id.clone(),
            onmounted: move |e| onmounted.call(e),
            ..merged,
            {children}
        }
    }
}

/// Native (Blitz) arm: unchanged from before this slice -- Blitz has no
/// popover-API support at all, so this stays the functional floor, a
/// plain, always-in-flow `div`. Blitz has no SSR/hydration lane for the
/// "duplicate attribute" divergence in the web arm's doc above to ever
/// bite, but the caller-wins-except-`id` merge is applied here too for the
/// same reason: keep both arms' semantics identical rather than have one
/// depend on spread order and the other not.
#[cfg(not(feature = "web"))]
#[component]
fn ToastRegionRendered(
    id: String,
    length: usize,
    attributes: Vec<Attribute>,
    onmounted: EventHandler<MountedEvent>,
    children: Element,
) -> Element {
    // Same `style`-clobber fix as the web arm above (dev-docs/backlog.md row
    // 44) -- kept in both arms for the reason this file's own "Native
    // (Blitz) arm" doc gives for the identical id/attribute-dedup choice:
    // both arms' semantics should agree by construction, not by accident of
    // which one a caller happens to exercise.
    let attributes: Vec<Attribute> = attributes.into_iter().filter(|a| a.name != "id").collect();
    let (caller_style, attributes) = crate::fold_style_attributes(attributes);
    let region_style = match caller_style {
        Some(caller_style) => format!("--toast-count: {length}; {caller_style}"),
        None => format!("--toast-count: {length}"),
    };
    let merged = merge_attributes(vec![
        attributes!(div {
            role: "region",
            aria_label: "{length} notifications",
            tabindex: "-1",
            style: "{region_style}",
        }),
        attributes,
    ]);

    rsx! {
        div {
            id,
            onmounted: move |e| onmounted.call(e),
            ..merged,
            {children}
        }
    }
}

/// The ordered list containing rendered toasts.
#[component]
pub fn ToastList(props: ToastListProps) -> Element {
    rsx! {
        ol {
            ..props.attributes,
            {props.children}
        }
    }
}

/// A list item wrapper for a rendered toast.
#[component]
pub fn ToastListItem(props: ToastListItemProps) -> Element {
    rsx! {
        li {
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`Toast`] component
#[derive(Props, Clone, PartialEq)]
pub struct ToastProps {
    /// The toast's handle (see [`ToastId`]).
    pub id: ToastId,
    /// The index of the toast in the list.
    pub index: usize,
    /// How many times the toast has been updated; restarts its auto-dismiss countdown.
    #[props(default)]
    pub revision: u32,
    /// The title of the toast.
    pub title: String,
    /// An optional description for the toast.
    pub description: Option<String>,
    /// The type of toast.
    pub toast_type: ToastType,
    /// Callback to handle the close action of the toast.
    pub on_close: Callback<MouseEvent>,
    /// Whether the toast is permanent (not auto-dismissed).
    #[props(default = false)]
    pub permanent: bool,
    /// The toast's action button, if any (see [`ToastAction`]).
    #[props(default)]
    pub action: Option<ToastAction>,

    /// The duration for which the toast is displayed.
    pub duration: Option<Duration>,

    /// Additional attributes to apply to the toast element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the toast element.
    #[props(default)]
    pub children: Option<Element>,
}

/// The props for the [`ToastContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ToastContentProps {
    /// Additional attributes to apply to the toast content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the toast content element.
    pub children: Element,
}

/// The props for the [`ToastTitle`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ToastTitleProps {
    /// Additional attributes to apply to the toast title element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Additional children for the toast title element.
    #[props(default)]
    pub children: Option<Element>,
}

/// The props for the [`ToastDescription`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ToastDescriptionProps {
    /// Additional attributes to apply to the toast description element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Additional children for the toast description element.
    #[props(default)]
    pub children: Option<Element>,
}

/// The props for the [`ToastCloseButton`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ToastCloseButtonProps {
    /// Additional attributes to apply to the toast close button.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the toast close button.
    #[props(default)]
    pub children: Option<Element>,
}

#[derive(Clone)]
struct ToastRenderCtx {
    toast_id: ToastId,
    label_id: String,
    description_id: String,
    /// The live record: the title, description, type and action follow every update. Falls
    /// back to the first render's snapshot in the instant between the record's removal and the
    /// toast's unmount.
    record: Memo<ToastRecord>,
    on_close: Callback<MouseEvent>,
}

/// # Toast
///
/// An individual toast notification with a message for the user. This is called automatically by the [`ToastProvider`] when a toast is added if you leave
/// the default `render_toast` callback.
///
/// If you call this component manually, it must be used inside a [`ToastProvider`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::toast::{ToastOptions, ToastProvider, use_toast};
/// use std::time::Duration;
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ToastProvider { ToastButton {} }
///     }
/// }
///
/// #[component]
/// fn ToastButton() -> Element {
///     let toast_api = use_toast();
///
///     rsx! {
///         button {
///             onclick: move |_| {
///                 toast_api
///                     .info(
///                         "Custom Toast".to_string(),
///                         ToastOptions::new()
///                             .description("Some info you need")
///                             .duration(Duration::from_secs(60))
///                     );
///             },
///             "Info (60s)"
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`Toast`] component defines the following data attributes you can use to control styling:
/// - `data-type`: The type of toast. Values are `success`, `error`, `warning`, `info`, or `loading`.
/// - `data-permanent`: Indicates if the toast is permanent. Values are `true` or `false`.
/// - `data-toast-even`: Present on even-indexed toasts for alternating styles.
/// - `data-toast-odd`: Present on odd-indexed toasts for alternating styles.
/// - `data-top`: Present on the topmost toast.
/// - `data-visible`: `true` while the toast is within the provider's `visible_toasts` cap, `false` while it waits hidden behind it (also when the pile is expanded).
/// - `data-removed`: `true` while a closed toast plays its exit animation (it is inert and no longer counts toward the pile); it unmounts once the 200 ms exit has finished (a short slack after it). Its list item carries the same `data-removed`, so the slot leaves the layout at once and the remaining toasts close the gap while it fades.
/// - `data-swiping`: `true` while a pointer drags the toast, `dismissed` while it flies out after a swipe.
///
/// While dragged, the toast sets `--toast-swipe-x` or `--toast-swipe-y` (px, toward the dismiss edge of the axis the drag took); map them to the `translate` property.
///
/// The [`Toast`] component renders toasts with the following css variables you can use to control styling:
/// - `--toast-index`: The index of the toast in the pile (closing toasts do not count), used for z-indexing and positioning.
/// - `--toast-height` (on the toast's list item, so the toast inherits it): the toast's own height in px, content plus padding and border, measured from the child marked `data-toast-body` (see `TOAST_SIZE_JS`). Absent until measured, and for a skin with no `data-toast-body`.
/// - `--front-toast-height` (on the list): the height of the front toast, the first one that is not closing. A collapsed pile sizes every toast behind the front one with it.
/// - `--toast-exit-ms`: How long a closed toast stays mounted (`200ms`); the exit transition should last that long.
#[component]
pub fn Toast(props: ToastProps) -> Element {
    let dom_id = use_unique_id();
    let id = use_memo(move || format!("toast-{dom_id}"));
    let label_id = format!("{id}-label");
    let description_id = format!("{id}-description");

    // Get the context at the top level of the component
    let ctx = use_context::<ToastCtx>();
    let snapshot = ToastRecord {
        id: props.id,
        revision: props.revision,
        title: props.title.clone(),
        description: props.description.clone(),
        toast_type: props.toast_type,
        duration: props.duration,
        permanent: props.permanent,
        action: props.action.clone(),
        closing: false,
    };
    let toast_key = props.id;
    let toasts = ctx.toasts;
    let record = use_memo(move || {
        toasts
            .read()
            .iter()
            .find(|t| t.id == toast_key)
            .cloned()
            .unwrap_or_else(|| snapshot.clone())
    });
    use_context_provider(|| ToastRenderCtx {
        toast_id: props.id,
        label_id: label_id.clone(),
        description_id: description_id.clone(),
        record,
        on_close: props.on_close,
    });

    // Swipe to dismiss (pointer events, threshold + velocity; see `use_toast_swipe`).
    let toasts_for_swipe = ctx.toasts;
    use_toast_swipe(id.into(), move || close_record(toasts_for_swipe, toast_key));

    // Derived here, from the record and the provider's cap, so no themed wrapper has to pass them:
    // `closing` while the exit animation plays, `visible` while the toast is within the cap.
    let closing = record.read().closing;
    let visible = props.index < (ctx.visible_toasts)();

    let children = props.children.unwrap_or_else(|| {
        rsx! {
            ToastContent {
                ToastTitle {}
                ToastDescription {}
            }
            ToastActionButton {}
            ToastCloseButton {}
        }
    });

    // Auto-dismissal. A child component rather than a hook here: whether a toast has a
    // countdown changes during its life (loading -> success), and a hook behind a condition
    // would change the hook order. Keyed on the duration and the revision, so an update
    // restarts the countdown with the new duration.
    let timer = match props.duration {
        Some(duration) if !props.permanent && props.toast_type != ToastType::Loading => {
            let key = format!("{}-{}", duration.as_millis(), props.revision);
            rsx! {
                ToastTimer { key: "{key}", id: props.id, duration }
            }
        }
        _ => rsx! {},
    };

    // `dx-toast-base-item` -- see `ensure_toast_base_styles`'s doc: the
    // plain marker class the web arm's engine-injected fallback stylesheet
    // selects on, giving a raw (unstyled-wrapper) toast a real, nonzero box
    // cross-engine (dev-docs/backlog.md row 44). Routed through
    // `merge_attributes` rather than a bare literal alongside `..props.
    // attributes` below -- a themed wrapper's own `class` (e.g.
    // `preview/src/components/toast/component.rs`'s `class: Styles::
    // dx_toast`) arrives inside `props.attributes` (the `#[props(extends =
    // GlobalAttributes)]` catch-all), and a second, bare `class` literal on
    // the same tag would be the identical duplicate-attribute hazard this
    // file's own "Attribute-override dedup" doc (above, on
    // `ToastRegionRendered`) already explains -- `merge_attributes`'s
    // concatenating `class` rule is what lets both live on the one
    // resulting attribute instead.
    // `style` needs the same fold the region above does, and for the same
    // reason (dev-docs/backlog.md row 44): a caller's own plain `style` arrives
    // in `props.attributes` under the *same* `("style", None)` key as this
    // element's own `--toast-index` literal, so spreading both onto one tag
    // makes `merge_attributes` treat them as a single collision and keep
    // only the caller's -- silently dropping the index the themed
    // stylesheet's `z-index: calc(var(--toast-count) - var(--toast-index))`
    // reads. No caller in this repo passes a per-toast `style` today, so
    // this is prevention rather than a live defect; it is fixed here anyway
    // because leaving the identical clobber on the sibling element of the
    // one row 44 fixes is exactly how that bug got two independent
    // instances in the first place. `--toast-index` goes first so a caller
    // can override any property but never drop the counter.
    let (caller_style, props_attributes) = crate::fold_style_attributes(props.attributes);
    let item_style = match caller_style {
        Some(caller_style) => format!(
            "--toast-index: {}; --toast-exit-ms: {}ms; {caller_style}",
            props.index,
            TOAST_EXIT.as_millis()
        ),
        None => format!(
            "--toast-index: {}; --toast-exit-ms: {}ms",
            props.index,
            TOAST_EXIT.as_millis()
        ),
    };
    let attributes = merge_attributes(vec![
        attributes!(div {
            class: "dx-toast-base-item",
            style: "{item_style}",
        }),
        props_attributes,
        // Owned by this component -- role/aria wiring/state must win over
        // a caller's own attributes (`dev-docs/backlog.md` row 93's
        // duplicate-attribute hazard).
        attributes!(div {
            role: "alertdialog",
            aria_labelledby: "{label_id}",
            aria_describedby: props.description.as_ref().map(|_| description_id),
            aria_modal: "false",
            tabindex: "0",
            "data-type": props.toast_type.as_str(),
            "data-permanent": props.permanent,
            "data-toast-even": (props.index % 2 == 0).then_some("true"),
            "data-toast-odd": (props.index % 2 == 1).then_some("true"),
            "data-top": (props.index == 0).then_some("true"),
            "data-visible": visible.to_string(),
            "data-removed": closing.then_some("true"),
            "inert": closing.then_some("true"),
        }),
    ]);

    rsx! {
        div {
            id,
            ..attributes,

            {children}
            {timer}
        }
    }
}

/// Counts a toast down and removes it. The countdown runs while the TAB is visible and the
/// toaster is not hovered or focused, and nothing else.
///
/// Hidden tab: the countdown is dropped and restarted in full on return, so a toast the user
/// could not see (hidden tab, minimized window) is still there when they come back. Hover or
/// focus inside the toaster: the countdown WAITS and resumes with the time that was left
/// (WCAG 2.2.1, timing adjustable; Base UI does the same). Remaining time is counted down in
/// `TICK` steps that only run while active, so a pause needs no clock. It is deliberately NOT
/// tied to the toast's position on the page (`use_motion`): a toast is a message the user is
/// waiting on, not decoration, and scrolling must not change how long it stays
/// (`crate::activity`, "Timers that carry meaning").
#[component]
fn ToastTimer(id: ToastId, duration: Duration) -> Element {
    const TICK: Duration = Duration::from_millis(50);
    let ctx = use_context::<ToastCtx>();
    let remove_toast = ctx.remove_toast;
    let paused = ctx.paused;
    let visible = crate::activity::use_document_visible();
    let mut remaining = use_hook(|| CopyValue::new(duration));
    use_effect(move || {
        if !visible() {
            remaining.set(duration);
        }
    });
    let running = use_memo(move || visible() && !paused());
    crate::interval::use_interval_while(running, TICK, move || {
        let left = remaining.peek().saturating_sub(TICK);
        remaining.set(left);
        if left.is_zero() {
            remove_toast.call(id);
        }
    });
    rsx! {}
}

/// Swipe to dismiss, with pointer events (mouse, touch and pen alike).
///
/// All of the gesture runs in the page (`listen`, so the listeners are removed with the
/// component): the drag writes `--toast-swipe-x` (horizontal) or `--toast-swipe-y` (vertical, toward the top or bottom edge) on the toast (the stylesheet maps it to the
/// `translate` property, so the stacking `scale` is untouched) and marks `data-swiping`, and
/// only the final decision crosses into Rust. A swipe dismisses when it travelled at least
/// `THRESHOLD` px toward the dismiss edge, or was a flick: at least `MIN_FLICK` px at
/// `VELOCITY` px/ms or more over the last 100 ms.
///
/// The dismiss direction is NOT a prop: it is the edge the toast is nearest to (the half of the
/// viewport the toast is in), so it follows wherever the stylesheet puts the toaster, and RTL
/// (`inset-inline-end` is the left edge there) with no second source of truth to keep in step.
/// A drag the other way is resisted to zero. A press that starts on a button or link is left
/// alone (the close and action buttons keep their click).
fn use_toast_swipe(dom_id: ReadSignal<String>, on_dismiss: impl FnMut() + 'static) {
    let mut on_dismiss = on_dismiss;
    let on_dismiss = use_callback(move |()| on_dismiss());
    crate::js_listener::use_js_listeners(move || {
        let id = dom_id.cloned();
        Some(
            crate::js_listener::JsListeners::install(TOAST_SWIPE_JS)
                .arg(id)
                .on_message(move |_: bool| on_dismiss.call(())),
        )
    });
}

/// The swipe gesture's script. See [`use_toast_swipe`].
const TOAST_SWIPE_JS: &str = r#"
const THRESHOLD = 45, MIN_FLICK = 10, VELOCITY = 0.11, WINDOW_MS = 100, SLOP = 3;
const id = await dioxus.recv();
const el = document.getElementById(id);
if (!el) return;
let drag = null;
const prop = (axis) => '--toast-swipe-' + axis;
const reset = () => {
  el.style.removeProperty('--toast-swipe-x');
  el.style.removeProperty('--toast-swipe-y');
  delete el.dataset.swiping;
  el.style.removeProperty('user-select');
};
const pos = (e, axis) => (axis === 'x' ? e.clientX : e.clientY);
// How far the pointer is from the press, toward the dismiss edge of `axis` (0 the other way).
const toward = (e, d, axis) => Math.max(0, (pos(e, axis) - d.start[axis]) * d.dir[axis]);
const end = (e) => {
  if (!drag || e.pointerId !== drag.pointer) return;
  const d = drag;
  drag = null;
  try { el.releasePointerCapture(e.pointerId); } catch (_) {}
  const axis = d.axis;
  if (!axis) { reset(); return; }
  const travelled = toward(e, d, axis);
  const recent = d.samples.filter((s) => e.timeStamp - s.t <= WINDOW_MS);
  const first = recent[0];
  const dt = first ? e.timeStamp - first.t : 0;
  const speed = dt > 0 ? ((pos(e, axis) - first[axis]) * d.dir[axis]) / dt : 0;
  const dismiss = e.type === 'pointerup' &&
    (travelled >= THRESHOLD || (travelled >= MIN_FLICK && speed >= VELOCITY));
  if (!dismiss) { reset(); return; }
  el.dataset.swiping = 'dismissed';
  const size = axis === 'x' ? el.offsetWidth : el.offsetHeight;
  el.style.setProperty(prop(axis), d.dir[axis] * (size + 48) + 'px');
  // The toast leaves the pile now (the Rust side marks it closing: its slot shrinks while it
  // flies out), not after the fly-out, so the others never wait for it and never jump.
  dioxus.send(true);
};
listen(el, 'pointerdown', (e) => {
  if (drag || (e.pointerType === 'mouse' && e.button !== 0)) return;
  if (e.target.closest('button, a, input, textarea, select, [role="button"]')) return;
  const rect = el.getBoundingClientRect();
  // The dismiss edge on each axis is the one the toast sits nearest to: the left or right edge
  // by the half of the viewport it is in (so RTL, where the toaster is on the left, flips by
  // itself), and the top or bottom edge likewise. Which axis a drag uses is decided by its
  // first movement.
  const dir = {
    x: rect.left + rect.width / 2 > innerWidth / 2 ? 1 : -1,
    y: rect.top + rect.height / 2 > innerHeight / 2 ? 1 : -1,
  };
  drag = {
    pointer: e.pointerId, axis: null, dir,
    start: { x: e.clientX, y: e.clientY },
    samples: [{ x: e.clientX, y: e.clientY, t: e.timeStamp }],
  };
  el.dataset.swiping = 'true';
  el.style.userSelect = 'none';
  try { el.setPointerCapture(e.pointerId); } catch (_) {}
});
listen(el, 'pointermove', (e) => {
  if (!drag || e.pointerId !== drag.pointer) return;
  drag.samples.push({ x: e.clientX, y: e.clientY, t: e.timeStamp });
  if (drag.samples.length > 32) drag.samples.shift();
  if (!drag.axis) {
    const ax = Math.abs(e.clientX - drag.start.x), ay = Math.abs(e.clientY - drag.start.y);
    if (Math.max(ax, ay) < SLOP) return;
    drag.axis = ax >= ay ? 'x' : 'y';
  }
  const axis = drag.axis;
  el.style.setProperty(prop(axis), toward(e, drag, axis) * drag.dir[axis] + 'px');
});
listen(el, 'pointerup', end);
listen(el, 'pointercancel', end);
"#;

/// The content wrapper inside a toast.
#[component]
pub fn ToastContent(props: ToastContentProps) -> Element {
    // Owned by this component -- role/aria wiring must win over a
    // caller's own attributes (`dev-docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    //
    // The live region: `status` (polite) for every type, `alert` (assertive) for an error.
    // The element is keyed on its role: a toast that moves from loading to error gets a NEW
    // `role="alert"` region (a region inserted with its content is announced; flipping the
    // `role` of a live one is not reliably), and a move between polite types keeps the same
    // `status` region whose text change is announced once. Either way it is the one toast.
    let role = use_context::<ToastRenderCtx>()
        .record
        .read()
        .toast_type
        .live_role();
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            role: role,
            aria_atomic: "true",
        }),
    ]);

    // A one-item keyed list: Dioxus diffs lists by key, so a new role replaces the element (a
    // `key` on a lone root element would be patched in place).
    rsx! {
        for r in [role] {
            div { key: "{r}", ..attributes.clone(), {props.children.clone()} }
        }
    }
}

/// The title element inside a toast.
#[component]
pub fn ToastTitle(props: ToastTitleProps) -> Element {
    let ctx = use_context::<ToastRenderCtx>();
    let children = props.children.unwrap_or_else(|| {
        let title = ctx.record.read().title.clone();
        rsx! { {title} }
    });

    // Owned by this component -- `id` is the labelledby target `Toast`'s
    // `aria-labelledby` reads back, so it must win over a caller's own
    // `id` rather than risk the SSR duplicate-attribute hazard
    // (`dev-docs/backlog.md` row 93 names this exact site).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            id: ctx.label_id.clone()
        }),
    ]);

    rsx! {
        div { ..attributes, {children} }
    }
}

/// The description element inside a toast.
#[component]
pub fn ToastDescription(props: ToastDescriptionProps) -> Element {
    let ctx = use_context::<ToastRenderCtx>();
    let id = ctx.description_id.clone();
    let Some(description) = ctx.record.read().description.clone() else {
        return rsx! {};
    };
    let children = props.children.unwrap_or_else(|| rsx! { {description} });

    // Owned by this component -- see `ToastTitle`'s identical
    // construction above (`dev-docs/backlog.md` row 93 names this exact site).
    let attributes = merge_attributes(vec![props.attributes, attributes!(div { id })]);

    rsx! {
        div { ..attributes, {children} }
    }
}

/// The close button inside a toast.
#[component]
pub fn ToastCloseButton(props: ToastCloseButtonProps) -> Element {
    let ctx = use_context::<ToastCtx>();
    let render_ctx = use_context::<ToastRenderCtx>();
    let children = props.children.unwrap_or_else(|| rsx! { "×" });

    // `aria-label`/`type="button"` are overridable defaults, not owned
    // state (construction policy) -- caller wins, so this is
    // `merge_attributes(vec![defaults, caller])`, not
    // `vec![caller, owned]` (`dev-docs/backlog.md` row 93).
    let attributes = merge_attributes(vec![
        attributes!(button {
            aria_label: "close",
            type: "button",
        }),
        props.attributes,
    ]);

    rsx! {
        button {
            onclick: move |e| {
                // Focus the region again after closing
                ctx.focus_region.call(());
                render_ctx.on_close.call(e);
            },
            ..attributes,
            {children}
        }
    }
}

/// Options for customizing the behavior of toasts dispatched from the [`Toasts`] context.
/// The props for [`ToastActionButton`].
#[derive(Props, Clone, PartialEq)]
pub struct ToastActionButtonProps {
    /// Additional attributes to apply to the button.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// What a toast's action button needs: its label and what activating it does. A themed wrapper
/// that renders its own button component (the preview's `Button`) reads this through
/// [`use_toast_action`] so activation is the same thing in every skin.
#[derive(Clone, Copy)]
pub struct ToastActionHandle {
    record: Memo<ToastRecord>,
    toast_id: ToastId,
    remove_toast: Callback<ToastId>,
    focus_region: Callback,
}

impl ToastActionHandle {
    /// The action's label.
    pub fn label(&self) -> String {
        self.record
            .read()
            .action
            .as_ref()
            .map(|a| a.label.clone())
            .unwrap_or_default()
    }

    /// Runs the action's callback, then dismisses the toast.
    pub fn activate(&self) {
        let action = self.record.peek().action.clone();
        if let Some(action) = action {
            action.on_click.call(());
        }
        self.focus_region.call(());
        self.remove_toast.call(self.toast_id);
    }
}

/// The action of the current toast, or `None` when it has none. Call inside a toast.
pub fn use_toast_action() -> Option<ToastActionHandle> {
    let render = use_context::<ToastRenderCtx>();
    let ctx = use_context::<ToastCtx>();
    render.record.read().action.as_ref()?;
    Some(ToastActionHandle {
        record: render.record,
        toast_id: render.toast_id,
        remove_toast: ctx.remove_toast,
        focus_region: ctx.focus_region,
    })
}

/// The toast's action button (see [`ToastAction`]): a real `<button>`, in the tab order,
/// activated by click, Enter or Space; it runs the action and dismisses the toast. Renders
/// nothing when the toast has no action.
#[component]
pub fn ToastActionButton(props: ToastActionButtonProps) -> Element {
    let Some(handle) = use_toast_action() else {
        return rsx! {};
    };
    let attributes = merge_attributes(vec![
        attributes!(button { type: "button" }),
        props.attributes,
    ]);
    rsx! {
        button {
            onclick: move |_| handle.activate(),
            ..attributes,
            {handle.label()}
        }
    }
}

/// Options for a new toast: description, duration, permanence and an action button.
#[derive(Clone, Default)]
pub struct ToastOptions {
    description: Option<String>,
    duration: Option<Duration>,
    permanent: bool,
    action: Option<ToastAction>,
}

impl ToastOptions {
    /// Create a new `ToastOptions` with an empty description, no duration, that is not permanent.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the description for the toast.
    pub fn description(mut self, description: impl ToString) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set the duration for the toast.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Set whether the toast is permanent.
    pub fn permanent(mut self, permanent: bool) -> Self {
        self.permanent = permanent;
        self
    }

    /// Adds an action button (e.g. "Undo"); activating it runs `on_click` and dismisses the
    /// toast.
    pub fn action(mut self, label: impl ToString, on_click: impl FnMut(()) + 'static) -> Self {
        self.action = Some(ToastAction::new(label, on_click));
        self
    }
}

/// A change to a live toast, for [`Toasts::update`]. Only what is set changes.
#[derive(Clone, Default)]
pub struct ToastUpdate {
    title: Option<String>,
    description: Option<Option<String>>,
    toast_type: Option<ToastType>,
    duration: Option<Duration>,
    permanent: Option<bool>,
    action: Option<Option<ToastAction>>,
}

impl ToastUpdate {
    /// An update that changes nothing yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the title.
    pub fn title(mut self, title: impl ToString) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Replace the description.
    pub fn description(mut self, description: impl ToString) -> Self {
        self.description = Some(Some(description.to_string()));
        self
    }

    /// Removes the description.
    pub fn clear_description(mut self) -> Self {
        self.description = Some(None);
        self
    }

    /// Change the type (e.g. `Loading` to `Success`).
    pub fn toast_type(mut self, toast_type: ToastType) -> Self {
        self.toast_type = Some(toast_type);
        self
    }

    /// Set the auto-dismiss duration.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Set whether the toast is permanent.
    pub fn permanent(mut self, permanent: bool) -> Self {
        self.permanent = Some(permanent);
        self
    }

    /// Replace the action button.
    pub fn action(mut self, label: impl ToString, on_click: impl FnMut(()) + 'static) -> Self {
        self.action = Some(Some(ToastAction::new(label, on_click)));
        self
    }

    /// Removes the action button.
    pub fn clear_action(mut self) -> Self {
        self.action = Some(None);
        self
    }
}

/// The three messages of [`Toasts::promise`]: shown while the future runs, and on how it ends.
/// `success` and `error` get the outcome, so the title can name it.
pub struct PromiseMessages<T, E> {
    loading: String,
    success: Box<dyn Fn(&T) -> String>,
    error: Box<dyn Fn(&E) -> String>,
}

impl<T, E> PromiseMessages<T, E> {
    /// Messages for [`Toasts::promise`]: `loading` while it runs, `success` and `error` for the outcome.
    pub fn new(
        loading: impl ToString,
        success: impl Fn(&T) -> String + 'static,
        error: impl Fn(&E) -> String + 'static,
    ) -> Self {
        Self {
            loading: loading.to_string(),
            success: Box::new(success),
            error: Box::new(error),
        }
    }
}

/// The toast context provided by the [`use_toast`] hook.
#[derive(Clone, Copy)]
pub struct Toasts {
    add_toast: AddToastCallback,
    toasts: Signal<VecDeque<ToastRecord>>,
}

impl Toasts {
    /// Send a toast to the associated [`ToastProvider`] with the given title, type, and options.
    /// Returns its handle for [`update`](Self::update) and
    /// [`close`](Self::close).
    pub fn show(&self, title: String, toast_type: ToastType, options: ToastOptions) -> ToastId {
        let id = ToastId::next();
        self.add_toast.call(ToastRecord {
            id,
            revision: 0,
            title,
            description: options.description,
            toast_type,
            duration: if options.permanent {
                None
            } else {
                options.duration
            },
            permanent: options.permanent,
            action: options.action,
            closing: false,
        });
        id
    }

    /// Changes a live toast in place (same element, same position in the stack). Returns whether
    /// anything was updated: an id whose toast is closed (or was trimmed by `max_toasts`) is a
    /// no-op, and since ids are never reused it cannot reach another toast.
    pub fn update(&self, id: ToastId, update: ToastUpdate) -> bool {
        let mut toasts = self.toasts;
        let Ok(mut toasts) = toasts.try_write() else {
            return false;
        };
        let Some(toast) = toasts.iter_mut().find(|t| t.id == id) else {
            return false;
        };
        if let Some(title) = update.title {
            toast.title = title;
        }
        if let Some(description) = update.description {
            toast.description = description;
        }
        if let Some(toast_type) = update.toast_type {
            toast.toast_type = toast_type;
        }
        if let Some(duration) = update.duration {
            toast.duration = Some(duration);
        }
        if let Some(permanent) = update.permanent {
            toast.permanent = permanent;
        }
        if let Some(action) = update.action {
            toast.action = action;
        }
        toast.revision = toast.revision.wrapping_add(1);
        true
    }

    /// Closes a toast. Closing a closed toast does nothing.
    pub fn close(&self, id: ToastId) {
        close_record(self.toasts, id);
    }

    /// Shows one loading toast and moves that SAME toast to success or error when `future`
    /// finishes (never a second toast). The live region announces the loading text, then the
    /// outcome once: polite (`role="status"`) for success, assertive (`role="alert"`) for an
    /// error. The future runs on the app root, so it finishes (and the toast settles) even if
    /// the component that started it unmounts. If the user closes the toast first, the
    /// settling update is a no-op.
    pub fn promise<T, E, F>(&self, future: F, messages: PromiseMessages<T, E>) -> ToastId
    where
        T: 'static,
        E: 'static,
        F: std::future::Future<Output = Result<T, E>> + 'static,
    {
        let id = self.loading(messages.loading.clone(), ToastOptions::new());
        let this = *self;
        // The task awaits the caller's future and ends with it; it must outlive the component
        // that started it (else a toast stays "loading" forever).
        // interval-ok: not a timer, bounded by the future
        crate::dioxus_core::spawn_forever(async move {
            let settle = match future.await {
                Ok(value) => ToastUpdate::new()
                    .toast_type(ToastType::Success)
                    .title((messages.success)(&value)),
                Err(error) => ToastUpdate::new()
                    .toast_type(ToastType::Error)
                    .title((messages.error)(&error)),
            };
            this.update(id, settle.clear_description().clear_action());
        });
        id
    }

    /// Create a new success toast with the given title and options.
    pub fn success(&self, title: String, options: ToastOptions) -> ToastId {
        self.show(title, ToastType::Success, options)
    }

    /// Create a new error toast with the given title and options.
    pub fn error(&self, title: String, options: ToastOptions) -> ToastId {
        self.show(title, ToastType::Error, options)
    }

    /// Create a new warning toast with the given title and options.
    pub fn warning(&self, title: String, options: ToastOptions) -> ToastId {
        self.show(title, ToastType::Warning, options)
    }

    /// Create a new info toast with the given title and options.
    pub fn info(&self, title: String, options: ToastOptions) -> ToastId {
        self.show(title, ToastType::Info, options)
    }

    /// A loading toast: spinner, never auto-dismissed until updated to another type.
    pub fn loading(&self, title: String, options: ToastOptions) -> ToastId {
        self.show(title, ToastType::Loading, options)
    }
}

/// # use_toast
///
/// The `use_toast` hook provides access to the [`Toast`] api from the nearest [`ToastProvider`] which lets you
/// dispatch toasts from anywhere in your component tree.
///
/// This must be called under a [`ToastProvider`] component.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::toast::{ToastOptions, ToastProvider, use_toast};
/// use std::time::Duration;
///
/// #[component]
/// fn ToastButton() -> Element {
///     let toast_api = use_toast();
///
///     rsx! {
///         button {
///             onclick: move |_| {
///                 toast_api
///                     .info(
///                         "Custom Toast".to_string(),
///                         ToastOptions::new()
///                             .description("Some info you need")
///                             .duration(Duration::from_secs(60))
///                             .permanent(false),
///                     );
///             },
///             "Info (60s)"
///         }
///     }
/// }
/// ```
pub fn use_toast() -> Toasts {
    use_hook(consume_toast)
}

/// Consume the toast context from the context
///
/// This must be called under a [`ToastProvider`] component.
pub fn consume_toast() -> Toasts {
    let ctx = consume_context::<ToastCtx>();
    Toasts {
        add_toast: ctx.add_toast,
        toasts: ctx.toasts,
    }
}
