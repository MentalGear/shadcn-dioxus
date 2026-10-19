#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::core::{current_scope_id, use_drop};
use dioxus::prelude::*;
#[cfg(not(feature = "web"))]
use dioxus::prelude::{asset, manganis, Asset};
use dioxus_core::AttributeValue::Text;
#[cfg(feature = "calendar")]
use time::OffsetDateTime;

use crate::js_listener::{use_js_listeners, JsListeners};

pub use dioxus_attributes;

pub mod accordion;
pub mod activity;
pub mod alert_dialog;
pub mod aspect_ratio;
pub mod avatar;
#[cfg(feature = "calendar")]
pub mod calendar;
pub mod carousel;
pub mod chart;
pub mod checkbox;
pub mod collapsible;
mod collection;
#[cfg(feature = "color-picker")]
pub mod color_picker;
pub mod combobox;
pub mod command;
pub mod context_menu;
#[cfg(feature = "calendar")]
pub mod date_picker;
pub mod dialog;
pub mod direction;
pub mod drag_and_drop_list;
pub mod drawer;
pub mod dropdown_menu;
pub mod hover_card;
mod hover_intent;
pub mod input_otp;
pub mod interval;
pub mod js_listener;
pub mod keyboard_inset;
pub mod label;
mod listbox;
pub mod menu_item;
mod menu_root;
mod menu_semantics;
mod menu_sub;
pub mod menubar;
pub mod message_scroller;
pub mod modality;
mod move_interaction;
#[cfg(feature = "router")]
pub mod navbar;
// Deliberately NOT gated on `feature = "router"`, unlike `navbar` above:
// `NavigationMenuLink` renders a plain `a[href]` itself rather than
// `dioxus_router::Link`, so this module has no router dependency -- see
// its own module doc.
pub mod navigation_menu;
mod pointer;
pub mod popover;
pub mod portal;
pub mod progress;
pub mod questionnaire;
pub mod radio_group;
pub mod resizable;
pub mod scroll_area;
mod scroll_lock;
pub mod select;
mod selectable;
mod selection;
pub mod separator;
mod sleep;
pub mod slider;
pub mod switch;
pub mod tabs;
pub mod tag_group;
#[cfg(test)]
mod test_support;
pub mod toast;
pub mod toggle;
pub mod toggle_group;
pub mod toolbar;
pub mod tooltip;
mod top_layer;
mod typeahead;
pub(crate) mod r#virtual;
pub mod virtual_grid;
pub mod virtual_list;

/// The vendored `focus-trap` JS bundle (`primitives/src/ts/focus-trap.ts`,
/// generated and committed by `scripts/build-js-bundle.sh`, checked by
/// `scripts/check-js-bundle.sh`; there is no build script). Native (Blitz)
/// modal overlays (`dialog.rs`/`alert_dialog.rs`/`popover.rs`'s
/// `#[cfg(not(feature = "web"))]` arms) are its only remaining
/// consumers -- every web-arm modal overlay now uses the native-dialog
/// engine (`showModal()`'s own focus trap/restore/inertness/top-layer),
/// completing the two-engine overlay architecture (docs/plan.md). `#[cfg]`
/// -gated on this crate's `web` feature (not `target_family = "wasm"` --
/// see `docs/recommended-implementations.md` Caveat 1 for why that axis was
/// corrected) so that neither the wasm client nor a host build with `web`
/// on (the fullstack SSG server) ever references, and therefore never
/// bundles, this asset at all -- see [`focus_trap_script`] (this file), the
/// only thing that ever names this constant.
#[cfg(not(feature = "web"))]
pub(crate) const FOCUS_TRAP_JS: Asset = asset!("/src/js/focus-trap.js");

/// Renders the `<script>` tag that loads [`FOCUS_TRAP_JS`] -- native
/// (Blitz) arm only. `dialog.rs`/`alert_dialog.rs`/`popover.rs`'s Root
/// components all call this instead of rendering `document::Script`
/// directly, so no build with this crate's `web` feature on ever mentions
/// `FOCUS_TRAP_JS` in its rendered tree at all (a plain `#[cfg]` on the tag
/// inside one shared `rsx!` call wouldn't get you that -- the *other*
/// branch's body would still need to name the feature-absent constant to
/// type-check; splitting into two whole function bodies, one per feature,
/// is what actually keeps the reference out of that build's compilation
/// unit). A no-op on the web arm:
/// every modal overlay's web-arm focus handling comes from the browser's
/// own `showModal()` now, so there is nothing to load there.
#[cfg(not(feature = "web"))]
pub(crate) fn focus_trap_script() -> Element {
    rsx! {
        document::Script {
            src: FOCUS_TRAP_JS,
            defer: true
        }
    }
}

/// See [`focus_trap_script`]'s doc -- the web arm's no-op counterpart.
#[cfg(feature = "web")]
pub(crate) fn focus_trap_script() -> Element {
    rsx! {}
}

/// Process-global counter behind [`allocate_unique_id`].
static NEXT_UNIQUE_ID: AtomicUsize = AtomicUsize::new(0);

/// Format the `n`th generated id for the given side.
///
/// Ids come in two disjoint namespaces: `dxc-{n}` (server-allocated) and
/// `dxc-c{n}` (anywhere else). A pure function so the prefix rule is unit
/// testable.
///
/// # Why two namespaces (the collision class this prevents)
///
/// In fullstack SSG the *server* counter is process-global and shared by
/// every concurrently prerendered page, so one page's SSR ids can be
/// anything (`dxc-200..dxc-330`, ...). Hydrated components reuse the
/// server's value via `use_server_cached`. But a component mounted on the
/// client *after* hydration has no server-cached value and allocates from
/// the *client* counter, which starts at 0. If both counters formatted into
/// the same `dxc-{n}` namespace, such a late id could equal an SSR id still
/// in the DOM -- two elements, one `id` -- and `getElementById` (hence every
/// aria wiring, anchor binding and `showModal()` lookup) resolves to the
/// wrong one. Observed: a hidden SSR checkbox input and the late-mounted
/// command dialog both had `id="dxc-229"`, so the command palette never
/// opened. Keeping client-allocated ids under a different prefix makes that
/// collision impossible by construction, whatever the counters' values.
fn format_unique_id(is_server: bool, n: usize) -> String {
    if is_server {
        format!("dxc-{n}")
    } else {
        format!("dxc-c{n}")
    }
}

/// Allocate a fresh id for *this* build side (see [`format_unique_id`]).
/// Only run on the server, on the client when nothing was server-cached for
/// the hook (a post-hydration mount), or in a non-fullstack build.
fn allocate_unique_id() -> String {
    #[allow(unused_mut, unused_assignments)]
    let mut is_server = false;
    // `ssr` is on for the fullstack server build (`dioxus/server` implies
    // it) and for plain SSR; off for every client build.
    server_only!(is_server = true);
    format_unique_id(is_server, NEXT_UNIQUE_ID.fetch_add(1, Ordering::Relaxed))
}

/// Generate a runtime-unique id.
///
/// On the server: `dxc-{n}`. On a hydrating client: exactly the server's
/// value (read from the server cache, no allocation). On a client with no
/// cached value (mounted after hydration) and in pure-CSR builds:
/// `dxc-c{n}`, which can never equal any server id -- see
/// [`format_unique_id`].
fn use_unique_id() -> Signal<String> {
    #[allow(unused_mut, unused_assignments)]
    let mut cached: Option<String> = None;
    fullstack! {
        // The closure runs on the server, and on the client only on a cache
        // miss -- so a hydrated component never touches the client counter.
        cached = Some(dioxus::prelude::use_server_cached(allocate_unique_id));
    }
    use_signal(move || cached.unwrap_or_else(allocate_unique_id))
}

// Elements can only have one id so if the user provides their own, we must use it as the aria id.
fn use_id_or<T: Clone + PartialEq + 'static>(
    mut gen_id: Signal<T>,
    user_id: ReadSignal<Option<T>>,
) -> Memo<T> {
    // First, check if we have a user-provided ID
    let has_user_id = use_memo(move || user_id().is_some());

    // If we have a user ID, update the gen_id in an effect
    use_effect(move || {
        if let Some(id) = user_id() {
            gen_id.set(id);
        }
    });

    // Return the appropriate ID
    use_memo(move || {
        if has_user_id() {
            user_id().unwrap()
        } else {
            gen_id.peek().clone()
        }
    })
}

/// A controlled-or-uncontrolled prop trio: external value signal,
/// fallback default signal, and change callback. Bundles the three
/// pieces that always travel together when forwarding props into
/// internal hooks like [`use_controlled`].
#[derive(Clone, Copy)]
pub(crate) struct Controlled<T: Clone + PartialEq + 'static> {
    pub(crate) value: ReadSignal<Option<T>>,
    pub(crate) default: ReadSignal<T>,
    pub(crate) on_change: Callback<T>,
}

/// Allows some state to be either controlled or uncontrolled.
pub fn use_controlled<T: Clone + PartialEq + 'static>(
    prop: ReadSignal<Option<T>>,
    default: T,
    on_change: Callback<T>,
) -> (Memo<T>, Callback<T>) {
    let mut internal_value = use_signal(|| prop.cloned().unwrap_or(default));
    let value = use_memo(move || prop.cloned().unwrap_or_else(&*internal_value));

    let set_value = use_callback(move |x: T| {
        internal_value.set(x.clone());
        on_change.call(x);
    });

    (value, set_value)
}

/// Run some cleanup code when the component is unmounted if the effect was run.
fn use_effect_cleanup<F: FnOnce() + 'static>(#[allow(unused)] cleanup: F) {
    client!(crate::dioxus_core::use_drop(cleanup))
}

/// Run some cleanup code when the component is unmounted if the effect was run.
fn use_effect_with_cleanup<F: FnMut() -> C + 'static, C: FnOnce() + 'static>(mut effect: F) {
    let mut cleanup = use_hook(|| CopyValue::new(None as Option<C>));
    use_effect(move || {
        if let Some(cleanup) = cleanup.take() {
            cleanup();
        }
        cleanup.set(Some(effect()));
    });
    client!(crate::dioxus_core::use_drop(move || {
        if let Some(cleanup) = cleanup.take() {
            cleanup();
        }
    }))
}

/// A stack of escape listeners to allow only the top-most listener to be called.
#[derive(Clone)]
struct EscapeListenerStack(Rc<RefCell<Vec<ScopeId>>>);

fn use_global_escape_listener(mut on_escape: impl FnMut() + Clone + 'static) {
    let scope_id = current_scope_id();
    let stack = use_hook(move || {
        // Get or create the escape listener stack
        let stack: EscapeListenerStack = try_consume_context()
            .unwrap_or_else(|| provide_context(EscapeListenerStack(Default::default())));
        // Push the current scope onto the stack
        stack.0.borrow_mut().push(scope_id);
        stack
    });
    // Remove the current scope id from the stack when we unmount
    use_drop({
        let stack = stack.clone();
        move || {
            let mut stack = stack.0.borrow_mut();
            stack.retain(|id| *id != scope_id);
        }
    });
    use_global_keydown_listener("Escape", move || {
        // Only call the listener if this component is on top of the stack
        let stack = stack.0.borrow();
        if stack.last() == Some(&scope_id) {
            on_escape();
        }
    });
}

fn use_global_keydown_listener(key: &'static str, on_escape: impl FnMut() + Clone + 'static) {
    use_js_listeners(move || {
        let mut on_escape = on_escape.clone();
        Some(
            JsListeners::install(
                "const targetKey = await dioxus.recv();
                listen(document, 'keydown', (event) => {
                    if (event.key === targetKey) {
                        event.preventDefault();
                        dioxus.send(true);
                    }
                });",
            )
            .arg(key)
            .on_message(move |_: bool| on_escape()),
        )
    });
}

/// Light-dismiss when pointerdown/focusin lands outside the element with the given `id`.
/// `id` should be the id of the popover/dialog root that contains every "inside" element.
fn use_outside_dismiss(
    id: impl Readable<Target = String> + Copy + 'static,
    on_dismiss: impl FnMut() + Clone + 'static,
) {
    use_js_listeners(move || {
        let mut on_dismiss = on_dismiss.clone();
        Some(
            JsListeners::install(
                "const id = await dioxus.recv();
                // A pointer press outside the root is always a dismiss, even when it
                // lands on an ancestor element that wraps the popover (e.g. a scroll
                // container around it).
                const onPointer = e => {
                    const root = document.getElementById(id);
                    if (root && !root.contains(e.target)) dioxus.send(true);
                };
                // Focus moving outside the root dismisses too (e.g. tabbing away), but
                // ignore focus that lands on an *ancestor* of the root. Clicking the
                // popover's own non-focusable background blurs the focused control and
                // the browser moves focus to the nearest focusable ancestor, which
                // still contains the popover — that is not a real focus-out.
                const onFocus = e => {
                    const root = document.getElementById(id);
                    if (root && !root.contains(e.target) && !e.target.contains(root)) dioxus.send(true);
                };
                listen(document, 'pointerdown', onPointer, true);
                listen(document, 'focusin', onFocus, true);",
            )
            .arg(id.cloned())
            .on_message(move |_: bool| on_dismiss()),
        )
    });
}

/// Returns the previous value of a reactive signal.
///
/// Ported from dignifiedquire/dx-components (MIT OR Apache-2.0),
/// `primitives/src/lib.rs` @ 5af3cc292559a0e8d73c7b9a827c4ca08ef34d99
/// (`use_previous`, matching upstream Radix's `usePrevious(value)`).
/// Adapted: none -- taken as-is.
///
/// On each render, if `value` has changed since the last observed value, the
/// previous value is stored and returned. The initial previous value equals
/// the initial `value`.
pub fn use_previous<T: Clone + PartialEq + 'static>(value: ReadSignal<T>) -> Memo<T> {
    let mut prev = use_signal(|| value.cloned());
    let mut last_seen = use_signal(|| value.cloned());

    use_memo(move || {
        let current = value.cloned();
        let seen = last_seen.cloned();
        if current != seen {
            prev.set(seen);
            last_seen.set(current);
        }
        prev.cloned()
    })
}

/// Refocus the trigger when a menu-family surface closes, unless something
/// outside caused the close.
///
/// Ported from dignifiedquire/dx-components (MIT OR Apache-2.0),
/// `primitives/src/lib.rs` @ 5af3cc292559a0e8d73c7b9a827c4ca08ef34d99
/// (`use_refocus_on_close_unless`). Adapted: none -- taken as-is; only the
/// per-component wiring of `interacted_outside` is new (docs/plan.md
/// Phase 3.1).
///
/// Matches Radix's `onCloseAutoFocus` on `DropdownMenuContent`/
/// `ContextMenuContent`/etc.: when `open` transitions from `true` to
/// `false`, focus is returned to the trigger element by id -- *unless*
/// `interacted_outside` is `true`, meaning the close was caused by the user
/// clicking or focusing something outside the menu, in which case focus
/// should stay wherever the user put it rather than being yanked back to
/// the trigger.
pub(crate) fn use_refocus_on_close_unless(
    open: Memo<bool>,
    trigger_id: Signal<String>,
    interacted_outside: ReadSignal<bool>,
) {
    let prev_open = use_previous(open.into());
    use_effect(move || {
        if prev_open() && !open() && !interacted_outside() {
            let id = trigger_id();
            // Never by script after a touch or pen (`data-dx-modality`, `modality.rs`): nothing was
            // keyboard-focused to give back, and a script-focused trigger rings on iOS.
            // focus-ok: restore focus to the trigger on close
            document::eval(&format!(
                "var e=document.getElementById('{id}');if(e&&!/^(touch|pen)$/.test(document.documentElement.getAttribute('data-dx-modality')||''))e.focus()"
            ));
        }
    });
}

/// Listens for the native `reset` event on the `<form>` that owns the element
/// with the given `id`, calling `on_reset` whenever the form resets.
///
/// The browser's own form-reset algorithm only restores a form control's own
/// DOM state (checkedness/selectedness derived from its `defaultChecked`/
/// `defaultSelected` content attribute) -- it has no way to know about, let
/// alone update, this crate's own Dioxus signals. Radix's form-participation
/// components (`Checkbox`, `RadioGroup`, `Select`, `Switch`) solve this the
/// same way: a `reset` listener on the hidden control's owning form that
/// pushes the component's visible/Rust state back to its default.
///
/// Looks up the owning form via `element.form` (present on `input`/`select`)
/// falling back to `element.closest('form')` for elements -- like a
/// `RadioGroup`'s container `<div>` -- that are not themselves form-associated.
fn use_form_reset_listener(
    id: impl Readable<Target = String> + Copy + 'static,
    on_reset: impl FnMut() + Clone + 'static,
) {
    use_js_listeners(move || {
        let mut on_reset = on_reset.clone();
        Some(
            JsListeners::install(
                "const id = await dioxus.recv();
                const el = document.getElementById(id);
                const form = el && (el.form || el.closest('form'));
                listen(form, 'reset', () => dioxus.send(true));",
            )
            .arg(id.cloned())
            .on_message(move |_: bool| on_reset()),
        )
    });
}

/// Drive `showModal()`/`close()` on the `<dialog>` element with the given
/// `id` from `open`, guarded by the element's own `.open` DOM property.
///
/// Docs/plan.md Phase 4.2. Ported *by construction*, not by copying code,
/// from `docs/phase4-spike-findings.md`'s `use_dialog_open_driver`
/// (experiment 2's fix for experiment 1's stranded-signal defect, see
/// `docs/recommended-implementations.md` Caveat 1). The guard (checking
/// `dialog.open` before calling either method) mirrors
/// [`crate::top_layer::use_popover_sync`]'s `:popover-open` check for the
/// identical reason: a redundant call throws `InvalidStateError` (spike
/// experiment 3b).
///
/// Never bind the `<dialog>` element's `open` attribute declaratively
/// alongside this hook in the same build -- spike experiment 3c found that
/// combination doesn't crash, it silently *skips* `showModal()` (Dioxus
/// commits the declarative attribute during render, before this effect
/// runs, so the guard's own condition is already false by the time it
/// checks). That is exactly why this hook only exists in the
/// `#[cfg(feature = "web")]` leaf of `dialog.rs`/`alert_dialog.rs`'s
/// component split (`docs/phase4-spike-findings.md` Construction B) -- the
/// `not(feature = "web")` arm never binds `open` as an attribute either,
/// since that arm renders a plain `div`, not a `<dialog>`, at all.
///
/// Gated on `feature = "web"`, not `target_family = "wasm"`, for the same
/// reason [`crate::top_layer::use_popover_sync`] is (see that function's
/// doc): `dioxus-desktop` is a non-wasm binary with a real, working webview
/// `eval` and belongs on this same arm, not on native/Blitz's no-op one --
/// and, confirmed by the 2026-09-01 production incident
/// (`docs/recommended-implementations.md` Caveat 1), the fullstack SSG
/// prerender is a *host* (non-wasm) binary that also belongs on this arm
/// whenever this crate's `web` feature is on.
///
/// ## Closing waits for the exit (backlog row 141; the modal-close regression)
///
/// `open` going `false` does **not** call `close()` on the spot. A native
/// `close()` drops the `<dialog>` out of the top layer *synchronously*, and
/// everything that made the panel a modal panel goes with it in that one frame: the
/// `::backdrop` scrim, the viewport as the containing block of its `position: fixed`
/// box, the `:modal` rules, an ancestor's `opacity` no longer being ignored. A
/// closing panel is still animating out, so each of those showed as a glitch --
/// worst on the home page, where every demo sits in a `.dx-component-card`
/// (`content-visibility: auto`, so layout + paint containment): out of the top
/// layer a fixed panel is positioned against, sized by and CLIPPED to its card, so a
/// right-hand Sheet snapped from 384x800 to 384x256 and a Drawer from 1265x346 to
/// 1105x205 mid-slide. `transition: overlay allow-discrete` papered over that
/// in Chromium alone, and only until the `overlay` transition ended (not in step
/// with the panel's own exit); Firefox and Safari had no keep-alive at all.
///
/// So this driver keeps the dialog a real, open modal -- same top layer, same
/// geometry -- until every animation or transition running on it (the panel's own
/// and its `::backdrop`'s) has settled, bounded by `DIALOG_EXIT_TIMEOUT_MS`, and only then
/// calls `close()`: the same construction [`crate::popover`]'s modal arm uses, now
/// on every engine and for every dialog-engine modal. A stylesheet therefore keys
/// its exit on the dialog's `data-state="closed"` (set by every caller of this
/// hook) and never needs an engine-specific keep-alive. With no animation at all
/// (no stylesheet, reduced motion to `0s`) the wait is empty and it closes at once.
///
/// A newer request (a reopen while closing, or a second close) bumps a per-element
/// generation, so a stale wait that wakes up later does nothing; Escape is routed
/// here through state by [`use_dialog_close_sync`]'s `cancel` handler instead of
/// being allowed to `close()` natively.
#[cfg(feature = "web")]
fn use_dialog_open_driver(
    id: impl Readable<Target = String> + Copy + 'static,
    open: impl Readable<Target = bool> + Copy + 'static,
) {
    // Installs `top_layer::ensure_top_layer_ink_styles`'s engine-injected
    // ink baseline -- see that function's own doc for the bug this closes
    // and why this hook (every native-`<dialog>`/`showModal()` consumer:
    // `dialog.rs`, `alert_dialog.rs`, `popover.rs`'s modal arm, and
    // therefore `Sheet`) is one of the three provably-exhaustive call sites
    // it lists.
    use_effect(crate::top_layer::ensure_top_layer_ink_styles);

    use_effect(move || {
        let want_open = open.cloned();
        let id = id.cloned();
        // `inject` is prepended into this exact script, not left to the
        // separate `use_effect(ensure_top_layer_ink_styles)` above -- see
        // that function's own doc (`top_layer.rs`), "First-paint window,"
        // for why relying on that other effect's `document::eval()` having
        // already run by the time *this* one calls `showModal()` is not
        // safe to assume (two separate `document::eval()` calls have no
        // guaranteed relative ordering against each other), and why
        // prepending -- one synchronous script, not two separately-ordered
        // ones -- is what actually closes the gap. Same construction as
        // `top_layer.rs`'s `use_popover_sync`/
        // `use_popover_shown_while_mounted`, this hook's two siblings in
        // the fix's three exhaustive call sites.
        let inject = crate::top_layer::TOP_LAYER_INK_STYLES_INJECT_JS;
        let modality = crate::modality::MODALITY_JS;
        let open_focus = crate::modality::OPEN_FOCUS_JS;
        let forward = DIALOG_CLOSE_FORWARD_JS;
        let timeout = DIALOG_EXIT_TIMEOUT_MS;
        document::eval(&format!(
            "{inject}
            {modality};
            {open_focus}
            {forward}
            const dialog = document.getElementById('{id}');
            if (!dialog) return;
            const gen = (dialog.__dxDialogGen = (dialog.__dxDialogGen || 0) + 1);
            if ({want_open}) {{
                // A reopen while the exit is still playing ends the closing state: the dialog is
                // still a real, open modal, so there is nothing to show again.
                dialog.__dxClosing = false;
                dialog.__dxActivate = false;
                dialog.__dxCancelAt = 0;
                if (dialog.open) return;
                // `dxOpenFocus` (`modality.rs`) remembers the opener (`dialog.__dxOpener`, so the
                // close can hand focus back the moment it starts, `DIALOG_CLOSE_FORWARD_JS`) and
                // makes the one decision where focus goes INSIDE the panel: the author's
                // `initial_focus`, else the browser default for a keyboard or mouse open and the
                // panel for a touch or pen open, never an editable element for touch. No listener
                // of ours reads :focus-visible.
                dxDialogForwardKeys(dialog);
                dxOpenFocus(dialog, () => dialog.showModal());
                return;
            }}
            if (!dialog.open) return;
            // The close has started: Enter/Space from here on belong to the opener, not to the
            // panel that is animating out (see `DIALOG_CLOSE_FORWARD_JS`).
            dialog.__dxClosing = true;
            if (dialog.__dxActivate) {{ dxDialogActivateOpener(dialog); return; }}
            requestAnimationFrame(async () => {{
                // The exit: whatever is animating or transitioning on the dialog itself or
                // its ::backdrop (a pseudo-element animation reports the originating
                // element as its target). `getAnimations()` flushes style first, so the
                // transitions this very `data-state` flip started are already in the list.
                // An infinite animation is never an exit, and a finished one is done.
                const exiting = () => document.getAnimations().filter((a) =>
                    a.effect && a.effect.target === dialog && a.playState !== 'finished'
                    && a.effect.getComputedTiming().endTime !== Infinity);
                const deadline = performance.now() + {timeout};
                // Re-read after each wave: an exit can chain (a transition that only
                // starts once another ends).
                for (let running = exiting(); running.length && performance.now() < deadline; running = exiting()) {{
                    await Promise.race([
                        Promise.allSettled(running.map((a) => a.finished)),
                        new Promise((resolve) => setTimeout(resolve, Math.max(0, deadline - performance.now()))),
                    ]);
                }}
                if (dialog.__dxDialogGen === gen && dialog.isConnected && dialog.open) {{
                    dialog.close();
                    // The native close just returned focus to the element focused before the
                    // open; `final_focus` (or a lost focus) is settled by the one close policy.
                    dxCloseFocus(dialog);
                }}
            }});"
        ));
    });
}

/// Row 223: a modal that is animating out is still a real modal, so the page behind it (the
/// trigger included) stays blocked and unfocusable until the native `close()` -- a `.focus()` on
/// the opener is a no-op, and `inert` on the dialog does not unblock it (measured). Enter, Escape,
/// Enter on a trigger therefore used to type its second Enter into the closing panel and lose the
/// reopen. The construction: from the moment the close starts (or a `cancel` has been seen but the
/// state has not flipped yet, up to 250 ms), an Enter or Space aimed at the closing dialog ends the
/// exit on the spot -- the real `close()`, which returns focus -- and activates the opener the
/// user was pressing it on. `__dxQuietClose` makes that early `close` event only release the
/// scroll lock: it must not write `open = false` after the opener's click has written `true`.
/// One listener per dialog element, on the element (it goes with the element).
#[cfg(feature = "web")]
const DIALOG_CLOSE_FORWARD_JS: &str = r#"
function dxDialogFinish(dialog) {
    dialog.__dxDialogGen = (dialog.__dxDialogGen || 0) + 1;
    dialog.__dxClosing = false;
    if (dialog.isConnected && dialog.open) {
        dialog.__dxQuietClose = true;
        dialog.close();
    }
}
function dxDialogActivateOpener(dialog) {
    const opener = dialog.__dxOpener;
    dialog.__dxActivate = false;
    dxDialogFinish(dialog);
    if (opener && opener.isConnected) {
        opener.focus({ preventScroll: true });
        opener.click();
    }
}
function dxDialogForwardKeys(dialog) {
    if (dialog.__dxKeysForwarded) return;
    dialog.__dxKeysForwarded = true;
    /* listener-ok: on the dialog element itself, which owns it and takes it along when removed */
    dialog.addEventListener('keydown', (e) => {
        if ((e.key !== 'Enter' && e.key !== ' ') || e.repeat || e.ctrlKey || e.metaKey || e.altKey) return;
        const closing = dialog.__dxClosing === true;
        const cancelled = !closing && !!dialog.__dxCancelAt && performance.now() - dialog.__dxCancelAt < 250;
        const opener = dialog.__dxOpener;
        if ((!closing && !cancelled) || !opener || !opener.isConnected) return;
        e.preventDefault();
        e.stopPropagation();
        if (closing) {
            dxDialogActivateOpener(dialog);
        } else {
            // Escape was pressed but the state has not flipped yet: do it as soon as it does.
            dialog.__dxActivate = true;
            setTimeout(() => { dialog.__dxActivate = false; }, 400);
        }
    }, true);
}
"#;

/// How long [`use_dialog_open_driver`] waits for a dialog's exit animations before
/// closing it anyway: the bound that keeps a stuck or very long animation from
/// holding an open modal (and its inert page) hostage.
#[cfg(feature = "web")]
const DIALOG_EXIT_TIMEOUT_MS: u32 = 1500;

/// Sync the `<dialog>` element's native `close` event -- fired on Escape's
/// default `cancel` action, a `::backdrop`/outside-click `close()` call we
/// drive ourselves (see `dialog.rs`'s `use_dialog_backdrop_dismiss`), a
/// `method="dialog"` form submission, or any other close, browser- or
/// script-driven alike -- back into `set_open`, so the Rust signal can never
/// strand.
///
/// This is the fix for the exact defect class
/// `docs/recommended-implementations.md` Caveat 1 documents for upstream's
/// first `<dialog>` (`b3f6de53`): a one-way `showModal()`/`close()` binding
/// with no listener on the browser's own `close` event, so a native Escape
/// left the `open` signal stranded at `true` and the dialog could never
/// reopen (reproduced by execution in `docs/phase4-spike-findings.md`
/// experiment 1; fixed the same way here as experiment 2's
/// `use_dialog_close_sync`). Same eval-channel shape as
/// `use_form_reset_listener` above and
/// [`crate::top_layer::use_popover_sync`]'s browser-to-signal half.
///
/// See [`use_dialog_open_driver`]'s doc for why this is `#[cfg]`-gated the
/// same way.
#[cfg(feature = "web")]
fn use_dialog_close_sync(
    id: impl Readable<Target = String> + Copy + 'static,
    set_open: Callback<bool>,
) {
    // The page scroll lock above this dialog, if there is one, lets go at the dialog's real
    // `close` -- after the exit animation, never at the `open` flip. See `scroll_lock.rs`.
    let modal_closed = crate::scroll_lock::use_modal_closed();
    use_js_listeners(move || {
        let modal_closed = modal_closed.clone();
        Some(
            JsListeners::install(
                "const id = await dioxus.recv();
                const dialog = document.getElementById(id);
                // Escape's `cancel` is routed through state instead of taking its default
                // action, an immediate native `close()`: that would drop the dialog out of
                // the top layer mid-exit (see `use_dialog_open_driver`), which instead closes
                // it once the exit has played. A controlled dialog whose owner refuses the
                // change also stays open, rather than closing natively behind its back.
                // An early close made by the driver to hand the opener back its Enter (row 223)
                // is quiet: the state is already false, and writing it again could land after
                // the opener's own click wrote true.
                listen(dialog, 'close', () => {
                    const quiet = dialog.__dxQuietClose === true;
                    dialog.__dxQuietClose = false;
                    dioxus.send(quiet ? 'closed' : 'close');
                });
                listen(dialog, 'cancel', (e) => {
                    e.preventDefault();
                    dialog.__dxCancelAt = performance.now();
                    dioxus.send('cancel');
                });",
            )
            .arg(id.cloned())
            .on_message(move |event: String| {
                if event != "closed" {
                    set_open.call(false);
                }
                if event != "cancel" {
                    modal_closed.notify();
                }
            }),
        )
    });
}

/// Backdrop-click dismiss for a web modal arm's `<dialog>` (`showModal()`).
///
/// `showModal()`'s `::backdrop` covers the entire viewport and owns
/// hit-testing everywhere while the dialog is open
/// (`docs/phase4-spike-findings.md` experiment 6: `elementFromPoint`
/// resolves to the `<dialog>` element at every coordinate) -- and every
/// pointer event over a `::backdrop` (or any other pseudo-element) is
/// dispatched with its `target` set to the host element, i.e. the `<dialog>`
/// itself. So the discriminator for "did this click land on the backdrop
/// (dismiss) or on the dialog's own visible box (don't dismiss)" can't be
/// `event.target` -- it's already the dialog element either way, including
/// for clicks that land in the dialog's own padding/gap areas where no
/// child happens to be. A bounding-rect check against the dialog's own
/// rendered box (the same technique MDN's `<dialog>` guide recommends for
/// this exact "click outside to close" pattern) tells the two apart
/// correctly regardless of `event.target`.
///
/// Originally `dialog.rs`-only (Phase 4.2, docs/plan.md); moved here so
/// `popover.rs`'s modal web arm (native-dialog Popover migration) can share
/// it with `alert_dialog.rs`/`dialog.rs` rather than reimplementing it --
/// the same trio as [`use_dialog_open_driver`]/[`use_dialog_close_sync`].
/// `use_outside_dismiss` itself is not reusable here, because a
/// `showModal()` backdrop click's `event.target` *is* the `<dialog>` element
/// (it contains itself), so `use_outside_dismiss`'s
/// `!root.contains(e.target)` check can never fire for it.
///
/// ## The nested-dialog false positive (live-site bug report, 2026-09-01)
///
/// The rect test alone is not a sufficient discriminator, because the
/// listener is bound directly on the `<dialog>` element and a native `click`
/// bubbles: a click on *any* descendant control -- not just the backdrop --
/// reaches this same handler. Two independent guards are required before the
/// rect test is even consulted, both checked in the shared predicate below:
///
/// - **`event.detail === 0`**: per UI Events, a `click` is dispatched "when
///   a pointing device button is pressed and released on an element" *or*
///   "when an element is activated" -- e.g. Enter/Space on a focused button
///   (`<https://www.w3.org/TR/uievents/#event-type-click>`). For the latter,
///   non-pointer case, `detail` is 0: no pointing-device button was
///   involved. In Chromium such a synthesized click also carries
///   `clientX`/`clientY` of `0, 0`, which the rect test misreads as "outside"
///   for any dialog not pinned at the viewport's top-left corner. Rejecting
///   `detail === 0` up front means a keyboard-activated control's bubbled
///   click is never even rect-tested.
/// - **`event.target === dialog`**: a `showModal()` backdrop click's target
///   is always the `<dialog>` element itself (see this function's doc
///   above), so a *real* backdrop click and a real click on the dialog's own
///   padding both have `event.target === dialog` -- the rect test alone
///   already tells those two apart correctly. But a click that bubbled up
///   from a descendant control carries that control as its target, never the
///   dialog -- so requiring `event.target === dialog` rejects every bubbled
///   click outright, independent of `detail`, without narrowing what counts
///   as a real backdrop click.
///
/// Without both guards, activating a control nested inside the dialog's own
/// content -- e.g. this crate's nested-`Dialog` demo's "Open Nested Dialog"
/// button -- bubbles a `detail === 0` click up to the *outer* dialog's own
/// listener, which (mis)reads it as an outside click and closes the outer
/// dialog. Because the nested `<dialog>` is a DOM descendant of the outer
/// one, the outer dialog's own close (`display: none` via its UA default
/// styling) then forces the still-open nested dialog out of the top layer,
/// which fires its own `close` event a moment later too -- the nested dialog
/// visibly opens and then immediately closes, with focus never durably
/// landing on anything inside it. See `docs/plan.md`/this crate's
/// `native-dialog.spec.ts` Rule 7 for the oracle rules this fixes.
///
/// ## The opening-gesture false positive (user device report, 2026-09-03)
///
/// User report (iOS Safari, touch): opening the home page's default
/// (modal, `is_modal` defaults to `true`) `Popover` -- "Show Popover" --
/// shows only its `::backdrop`; the popover itself flashes and disappears.
/// This hook is a modal `Popover`/`Dialog`'s only dismissal path -- the web
/// arm calls neither `use_global_escape_listener` nor `use_outside_dismiss`
/// (the browser's own `showModal()` already supplies focus trap/restore and
/// inertness; this hook is what supplies backdrop-click dismiss) -- so a
/// dismiss firing on the *opening* tap itself matches the report exactly.
///
/// Reproduction attempted (this session): touch-emulated (`hasTouch`,
/// `locator.tap()`) opens of this exact popover, in this sandbox's
/// Chromium, keep it open -- confirmed by execution, no self-dismiss. That
/// is expected, not a sign there is nothing to fix: this listener is bound
/// directly on the `<dialog>` element, and the button that opens it is a
/// *sibling* of that dialog, not an ancestor -- the one native `click`
/// event dispatched for the opening tap has `event.target` fixed to the
/// button for its entire dispatch (per DOM event dispatch, a event's
/// `target` is set once and does not change mid-dispatch even if the
/// listener that runs partway through synchronously shows the dialog), and
/// it never reaches this handler at all, on any engine that dispatches
/// exactly one `click` per tap. iOS Safari's touch-to-click synthesis is
/// the documented exception: WebKit computes a synthesized click's hit-test
/// target lazily, and a DOM mutation between `touchend` and that synthesis
/// (exactly `showModal()`, moving this same coordinate under the freshly
/// shown `::backdrop`) can retarget the synthesized click at whatever now
/// sits there -- the dialog itself, i.e. `event.target === dialog`, passing
/// this hook's own discriminator and landing at the *trigger's* screen
/// position, which sits outside a trigger-anchored (not viewport-centered;
/// see `popover.rs`'s module doc) modal `Popover`'s own rendered box. This
/// is engine behavior this sandbox cannot exercise (no such click-retarget
/// quirk exists in this Chromium build to reproduce against), so this
/// remains device-only, per this session's own instruction not to claim a
/// reproduction that was not actually achieved.
///
/// Fixed by construction regardless: `dialog.addEventListener('click', ...)`
/// is deferred by one `requestAnimationFrame`, so the very click that opened
/// this dialog -- and any same-gesture follow-up event an engine might
/// synthesize immediately after it -- can never reach this listener at all,
/// on any engine, by construction rather than by guessing at Safari's exact
/// timing. A *separate*, later dismiss click still works exactly as before:
/// one frame (~16ms) is far shorter than the time a real second tap, or a
/// Playwright `.click()`'s own actionability/round-trip overhead, ever
/// takes to reach the browser. `cancelAnimationFrame` on cleanup covers the
/// case where the dialog closes (unmounts, see `popover.rs`'s "Two
/// execution-confirmed bugs" -- though this specific content has no exit
/// animation) before the deferred frame ever fires, so the listener is
/// never attached to an element already gone from the DOM.
///
/// ## The drag false positive (owner report, 2026-10-05: "dragging UP on a bottom
/// Drawer's handle closes it")
///
/// A `click` is dispatched on the nearest common ancestor of where the press and
/// the release landed. A press on the Drawer handle (or on text, or in an input)
/// and a release over the `::backdrop` above the panel have the dialog itself as
/// that ancestor, so the release produced a `click` with `target === dialog` and
/// coordinates outside the dialog's box -- indistinguishable, by the checks above,
/// from a real backdrop click. Every drag that began inside a dialog and ended
/// outside it (a text selection, a slider, a Drawer pulled the wrong way) closed
/// it. A dismissal is a *click on the backdrop*, so the press must have started
/// there too: a `pointerdown` whose target is the dialog and which landed outside
/// its box arms the next `click`, and any other press disarms it. Not tied to the
/// Drawer: this is the shared hook for Dialog, Sheet, Drawer, CommandDialog and
/// the modal Popover, so the whole class is closed here.
#[cfg(feature = "web")]
fn use_dialog_backdrop_dismiss(
    id: impl Readable<Target = String> + Copy + 'static,
    on_dismiss: impl FnMut() + Clone + 'static,
) {
    use_js_listeners(move || {
        let mut on_dismiss = on_dismiss.clone();
        Some(
            JsListeners::install(
                "const id = await dioxus.recv();
                const dialog = document.getElementById(id);
                const outside = (x, y) => {
                    const rect = dialog.getBoundingClientRect();
                    return !(x >= rect.left && x <= rect.right
                        && y >= rect.top && y <= rect.bottom);
                };
                // Whether the press that this click completes began on the backdrop --
                // see this hook's doc, 'The drag false positive'.
                let pressedOnBackdrop = false;
                const onDown = (e) => {
                    pressedOnBackdrop = e.target === dialog && outside(e.clientX, e.clientY);
                };
                const onClick = (e) => {
                    const startedOnBackdrop = pressedOnBackdrop;
                    pressedOnBackdrop = false;
                    // Ignore a click that isn't pointer-originated (a keyboard
                    // Enter/Space activation) and any click that bubbled up from
                    // a descendant control rather than landing on the dialog
                    // element itself -- see this hook's doc comment.
                    if (e.detail === 0 || e.target !== dialog) return;
                    if (startedOnBackdrop && outside(e.clientX, e.clientY)) dioxus.send(true);
                };
                // Deferred by one frame -- see this hook's doc, 'The
                // opening-gesture false positive' -- so the tap that opened
                // this dialog (still finishing its own dispatch, or a
                // same-gesture synthetic follow-up some engines emit) can never
                // reach this listener. `cleanup` cancels a frame that has not
                // fired yet; `listen` itself ignores one that fires late.
                const raf = requestAnimationFrame(() => {
                    listen(dialog, 'pointerdown', onDown);
                    listen(dialog, 'click', onClick);
                });
                cleanup(() => cancelAnimationFrame(raf));",
            )
            .arg(id.cloned())
            .on_message(move |_: bool| on_dismiss()),
        )
    });
}

/// Whether a completed (or aborted) close-animation cycle should still write
/// its result to `show_in_dom`.
///
/// Each run of [`use_animated_open`]'s effect -- whether it opens or closes --
/// bumps a generation counter before spawning its async work. A cycle is
/// stale, and its result must be dropped, once a newer cycle has started:
/// that newer cycle already owns `show_in_dom` (an open cycle sets it
/// synchronously; a closing cycle will set it when its own animation
/// settles), so applying a superseded cycle's result would clobber fresher
/// state with stale data. A cycle that is still current -- including one
/// whose animation was aborted by something other than a new open/close,
/// e.g. a script directly cancelling the animation -- must still apply, or
/// the element leaks in the DOM forever with no cycle left to unmount it.
fn should_apply_animation_result(spawned_generation: u64, current_generation: u64) -> bool {
    spawned_generation == current_generation
}

fn use_animated_open(
    id: impl Readable<Target = String> + Copy + 'static,
    open: impl Readable<Target = bool> + Copy + 'static,
) -> impl Fn() -> bool + Copy {
    // Show in dom is a few frames behind the open signal to allow for the animation to start.
    // If it does start, we wait for the animation to finish before showing removing the element from the DOM.
    let mut show_in_dom = use_signal(|| false);

    // Bumped at the top of every effect run (open or close) so a closing
    // task still in flight when a newer cycle starts can tell it is stale.
    // Written through `.write()` / read through `.peek()` only -- never
    // `.read()` -- so this effect never subscribes to its own counter.
    let mut generation = use_signal(|| 0u64);

    use_effect(move || {
        *generation.write() += 1;
        let my_generation = *generation.peek();

        let open = open.cloned();
        if open {
            show_in_dom.set(open);
        } else {
            spawn(async move {
                let id = id.cloned();
                let mut eval = dioxus::document::eval(
                    r#"const id = await dioxus.recv();
                    await new Promise(resolve => requestAnimationFrame(resolve));
                    const element = document.getElementById(id);
                    if (!element) {
                        dioxus.send(true);
                        return;
                    }
                    const anims = element.getAnimations();
                    if (anims.length > 0) {
                        // Hold the element in the DOM for an extra ~250ms after
                        // the close animation finishes so external observers
                        // (e.g. Playwright polling) reliably see the
                        // data-state="closed" element before it unmounts. The
                        // element is opacity:0 / pointer-events:none here, so
                        // the user sees nothing.
                        const hold = () => new Promise(r => setTimeout(r, 250));
                        Promise.all(anims.map((a) => a.finished))
                            .then(hold)
                            .then(() => dioxus.send(true))
                            .catch(() => {
                                // Animation aborted -- most often because a
                                // newer open/close cycle re-triggered it, but
                                // possibly because something else (e.g. a
                                // script) cancelled it directly. Always send
                                // so this task's recv() completes either way;
                                // the generation check on the Rust side is
                                // what decides whether the result still
                                // applies.
                                dioxus.send(false);
                            });
                    } else {
                        dioxus.send(true);
                    }"#,
                );
                let _ = eval.send(id);
                let _ = eval.recv::<bool>().await;

                if should_apply_animation_result(my_generation, *generation.peek()) {
                    show_in_dom.set(open);
                }
            });
        }
    });

    move || show_in_dom()
}

#[cfg(test)]
mod use_animated_open_tests {
    use super::should_apply_animation_result;

    #[test]
    fn current_generation_applies() {
        assert!(should_apply_animation_result(3, 3));
    }

    #[test]
    fn superseded_generation_is_skipped() {
        // A newer cycle started (generation moved on) while this one was
        // still awaiting its animation -- applying it now would clobber the
        // newer cycle's state.
        assert!(!should_apply_animation_result(3, 4));
    }
}

/// The side where the content will be displayed relative to the trigger
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentSide {
    /// The content will appear above the trigger
    Top,
    /// The content will appear to the right of the trigger
    Right,
    /// The content will appear below the trigger
    Bottom,
    /// The content will appear to the left of the trigger
    Left,
}

impl ContentSide {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Right => "right",
            Self::Bottom => "bottom",
            Self::Left => "left",
        }
    }
}

/// The alignment of the content relative to the trigger
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentAlign {
    /// The content will be aligned to the start of the trigger
    Start,
    /// The content will be centered relative to the trigger
    Center,
    /// The content will be aligned to the end of the trigger
    End,
}

impl ContentAlign {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

#[cfg(feature = "calendar")]
pub(crate) trait LocalDateExt {
    /// A small extension method function to get the local date with a fallback to UTC date if this fails
    fn now_local_date() -> time::Date;
}

#[cfg(feature = "calendar")]
impl LocalDateExt for time::OffsetDateTime {
    fn now_local_date() -> time::Date {
        OffsetDateTime::now_local()
            .map(|x| x.date())
            .unwrap_or_else(|_| time::UtcDateTime::now().date())
    }
}

/// Merge multiple attribute vectors.
///
/// Rules:
/// - Later lists win for the same (name, namespace) pair.
/// - `class` is concatenated with a single space separator (trimmed); the volatility flag is OR-ed.
/// - `style` is folded the same way, as CSS declarations: earlier declarations first, then the
///   later ones (each side trimmed, a missing `;` between them supplied), so a later list can
///   override individual properties but never silently drops the earlier list's other
///   declarations. This is what keeps a primitive's own `anchor-name: --dxa-..;` alive when a
///   caller adds `style: "border-radius: 9999px;"` -- including the common case where both
///   sit in ONE incoming list because Dioxus appends a component call's ad-hoc `style:` key
///   after its explicit `attributes` (`Button { style: "..", attributes }`). A later
///   `dioxus_core::AttributeValue::None` or empty `style` (a themed wrapper threading an unset `style`
///   prop through) is ignored instead of erasing the earlier one.
/// - Event listeners of the same name are CHAINED, not replaced, the way Radix's
///   `composeEventHandlers` (`packages/core/primitive/src/primitive.tsx`) does it: the LATER
///   list is the caller's, so its listener runs first, and the earlier list's (the
///   component's own, e.g. a close button's "close") runs after it unless the caller's called
///   `prevent_default()` on the event. Both receive the same event (shared `stop_propagation` /
///   `prevent_default`). A callback that is identical in both lists (`==`) runs once. A later
///   `AttributeValue::None` still removes an earlier listener, and a listener replaces a
///   non-listener value. (An event already prevented before the caller's listener ran does not
///   skip the component's: only a `prevent_default` made BY the caller's listener does.)
/// - Other attributes are overwritten by the last occurrence.
///
/// Only a plain `style` (namespace `None`) is folded here; CSS set the shorthand way
/// (`padding: ".."`, namespace `Some("style")`) is a different attribute name and is handled by
/// [`fold_style_attributes`].
pub fn merge_attributes(mut lists: Vec<Vec<Attribute>>) -> Vec<Attribute> {
    let mut merged = Vec::new();
    // The inputs are usually sorted by name, so we can do a k-way merge cheaply
    for list in &mut lists {
        list.sort_by_key(|a| a.name);
    }
    let mut iters: Vec<_> = lists
        .into_iter()
        .map(|l| l.into_iter().peekable())
        .collect();

    loop {
        // Find the minimum name among all current heads
        let min_name = iters
            .iter_mut()
            .filter_map(|it| it.peek().map(|a| a.name))
            .min();

        let Some(min_name) = min_name else {
            break;
        };

        // Collect all attributes with this name, grouped by namespace
        let mut by_namespace: Vec<Attribute> = Vec::new();

        for iter in &mut iters {
            while iter.peek().map(|a| a.name) == Some(min_name) {
                let attr = iter.next().unwrap();
                if let Some(existing) = by_namespace
                    .iter_mut()
                    .find(|a| a.namespace == attr.namespace)
                {
                    if attr.name == "class" {
                        let was_volatile = existing.volatile;
                        *existing = match (&existing.value, &attr.value) {
                            (Text(a), Text(b)) => Attribute {
                                name: attr.name,
                                namespace: attr.namespace,
                                volatile: was_volatile || attr.volatile,
                                value: Text(join_class(a, b)),
                            },
                            _ => attr,
                        };
                    } else if attr.name == "style" {
                        fold_style_into(existing, attr);
                    } else if let (
                        dioxus_core::AttributeValue::Listener(earlier),
                        dioxus_core::AttributeValue::Listener(later),
                    ) = (&existing.value, &attr.value)
                    {
                        // Two listeners for one event: both run (the later, caller's first, then
                        // the earlier unless the caller prevented default), never one silently
                        // replacing the other. The same callback arriving twice (one
                        // `attributes` Vec reaching a merge through two paths) runs once.
                        if earlier != later {
                            existing.value = chain_listeners(earlier.clone(), later.clone());
                        }
                    } else {
                        *existing = attr;
                    }
                } else {
                    by_namespace.push(attr);
                }
            }
        }

        merged.extend(by_namespace);
    }

    merged
}

/// One listener that runs `later` (the caller's) and then `earlier` (the component's own),
/// skipping `earlier` when `later` prevented the event's default (Radix `composeEventHandlers`).
///
/// Every listener an element attribute can carry (`onclick`, `onkeydown`, `onmounted`, ...) is
/// built by `dioxus-html` as a `ListenerCallback` over `Event<PlatformEventData>` (the typed
/// `MouseEvent` etc. is derived inside it), so the chained listener can be written once, without
/// knowing the event's type.
fn chain_listeners(
    earlier: dioxus_core::ListenerCallback,
    later: dioxus_core::ListenerCallback,
) -> dioxus_core::AttributeValue {
    dioxus_core::AttributeValue::listener(
        move |event: dioxus_core::Event<dioxus::html::PlatformEventData>| {
            let was_enabled = event.default_action_enabled();
            later.call(event.clone().into_any());
            if was_enabled && !event.default_action_enabled() {
                return;
            }
            earlier.call(event.into_any());
        },
    )
}

/// Pull every attribute that contributes to the rendered `style` out of
/// `attrs`, combine them into one CSS text string (in the order found), and
/// return `(combined_style, everything_else)`.
///
/// Exists for exactly one situation `merge_attributes` cannot fix on its
/// own: a component that has its own fixed CSS as one literal
/// `style: "a: b; c: d;"` attribute (namespace `None`), spread alongside a
/// caller's `attributes` that may carry CSS set the *shorthand* way
/// (`padding: "1rem"`, `cursor: "pointer"`, ...; each becomes its own
/// `Attribute` with namespace `Some("style")`). `dioxus-ssr`'s renderer
/// (`render_template`'s `Segment::Attr`/`Segment::StyleMarker` handling)
/// only accumulates and combines attributes with `namespace == Some("style")`
/// into the single served `style="..."` -- a plain `name == "style"`,
/// `namespace == None` attribute is a completely separate code path and is
/// written out as its own, second `style="..."` (confirmed by execution:
/// `ContextMenuTrigger`'s fixed touch-suppression CSS alongside the preview
/// app's themed wrapper's shorthand `padding`/`background`/etc. produced
/// exactly this — two `style` attributes on one tag, WHATWG HTML's
/// duplicate-attribute parse error, `docs/conformance-harness.md`
/// hydration-parity Rule 4). `merge_attributes` cannot help either: it
/// dedupes by `(name, namespace)`, and these two forms use different
/// `name`s (`"style"` vs `"padding"`/`"background"`/...), so from its
/// point of view they are simply different attributes, not a collision.
/// Call this on a component's own incoming `attributes` before deciding
/// its own literal `style` string, fold the two together in Rust, and pass
/// the returned `everything_else` on to `merge_attributes`/the `rsx!` call
/// instead of the original list, so at most one `style` is ever built.
pub(crate) fn fold_style_attributes(attrs: Vec<Attribute>) -> (Option<String>, Vec<Attribute>) {
    let mut style = String::new();
    let mut rest = Vec::with_capacity(attrs.len());
    for attr in attrs {
        let is_style_shorthand = attr.namespace == Some("style");
        let is_plain_style = attr.namespace.is_none() && attr.name == "style";
        if is_style_shorthand || is_plain_style {
            if let Text(value) = &attr.value {
                if !value.is_empty() {
                    if is_style_shorthand {
                        style.push_str(attr.name);
                        style.push(':');
                        style.push_str(value);
                        style.push(';');
                    } else {
                        style.push_str(value.trim());
                        if !style.ends_with(';') {
                            style.push(';');
                        }
                    }
                }
            }
        } else {
            rest.push(attr);
        }
    }
    (if style.is_empty() { None } else { Some(style) }, rest)
}

/// Whether `attributes` already gives its element its own accessible name
/// (a non-empty `aria-label` or `aria-labelledby`).
///
/// Shared by every popup-content component that defaults its own
/// `aria-labelledby` from its trigger's id (`DropdownMenuContent`,
/// `ContextMenuContent`, `MenubarContent`, `SelectList`) so that default
/// never fights a caller-supplied name -- `SelectList`'s own doc example
/// demonstrates `SelectList { aria_label: "...", ... }` as a supported
/// override, and ARIA's own precedence rules mean an unconditional
/// `aria-labelledby` would otherwise take priority over that `aria-label`
/// and silently shadow it.
///
/// Deliberately more than a name-presence check: `#[props(extends =
/// GlobalAttributes)]` lets a themed wrapper thread an `aria_label` prop
/// through unconditionally (`Some("")`/absent or not), so an attribute
/// named `aria-label` can be *present* in `attributes` with an
/// empty/`dioxus_core::AttributeValue::None` value even when no real caller ever
/// supplied one (confirmed by execution: this crate's own themed `Select`
/// wrapper, `preview/src/components/select/component.rs`, does exactly
/// this). Only a genuinely non-empty text value counts as caller-supplied.
pub(crate) fn has_own_accessible_name(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|a| {
        (a.name == "aria-label" || a.name == "aria-labelledby")
            && match &a.value {
                Text(s) => !s.is_empty(),
                dioxus_core::AttributeValue::None => false,
                // Any other concrete (non-text, non-empty) value counts as
                // caller-supplied.
                _ => true,
            }
    })
}

/// Fold a later `style` attribute into the earlier `existing` one (see `merge_attributes`).
fn fold_style_into(existing: &mut Attribute, attr: Attribute) {
    use dioxus_core::AttributeValue::None as NoValue;
    match (&existing.value, &attr.value) {
        (Text(a), Text(b)) => {
            let value = join_style(a, b);
            existing.volatile |= attr.volatile;
            existing.value = Text(value);
        }
        // An unset/empty later `style` must not erase the earlier one.
        (_, NoValue) => {}
        // Nothing to fold into: the later value wins, as for any other attribute.
        _ => *existing = attr,
    }
}

/// Concatenate two CSS declaration lists, `a` first so `b` can override single properties.
fn join_style(a: &str, b: &str) -> String {
    let (a, b) = (a.trim(), b.trim());
    match (a.is_empty(), b.is_empty()) {
        (true, _) => b.to_string(),
        (_, true) => a.to_string(),
        _ if a.ends_with(';') => format!("{a} {b}"),
        _ => format!("{a}; {b}"),
    }
}

fn join_class(a: &str, b: &str) -> String {
    let (a, b) = (a.trim(), b.trim());
    if !a.is_empty() && !b.is_empty() {
        format!("{a} {b}")
    } else {
        format!("{a}{b}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attr(name: &'static str, value: &str) -> Attribute {
        Attribute {
            name,
            namespace: None,
            volatile: false,
            value: Text(value.to_string()),
        }
    }

    fn get_value(attr: &Attribute) -> &str {
        match &attr.value {
            Text(s) => s,
            _ => panic!("expected Text"),
        }
    }

    #[test]
    fn merge_empty_lists() {
        let result = merge_attributes(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn merge_single_list() {
        let result = merge_attributes(vec![vec![attr("a", "1"), attr("b", "2")]]);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "a");
        assert_eq!(result[1].name, "b");
    }

    #[test]
    fn merge_preserves_sorted_order() {
        let result = merge_attributes(vec![
            vec![attr("a", "1"), attr("c", "3")],
            vec![attr("b", "2"), attr("d", "4")],
        ]);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].name, "a");
        assert_eq!(result[1].name, "b");
        assert_eq!(result[2].name, "c");
        assert_eq!(result[3].name, "d");
    }

    #[test]
    fn later_list_overwrites() {
        let result = merge_attributes(vec![vec![attr("a", "first")], vec![attr("a", "second")]]);
        assert_eq!(result.len(), 1);
        assert_eq!(get_value(&result[0]), "second");
    }

    /// `style` folds the way `class` does (declarations concatenated, earlier first), so a
    /// primitive's own `anchor-name`/`position-anchor` binding survives a caller-supplied
    /// `style` instead of being replaced by it (the avatar dropdown demo's trigger rendered at
    /// viewport (0,0) because `Button { style: "border-radius: 9999px;", attributes }` replaced
    /// the trigger's anchor-name style).
    #[test]
    fn style_attribute_is_folded_not_overwritten() {
        let result = merge_attributes(vec![
            vec![attr("style", "anchor-name: --dxa-x;")],
            vec![attr("style", "color: red;")],
        ]);
        assert_eq!(result.len(), 1);
        assert_eq!(get_value(&result[0]), "anchor-name: --dxa-x; color: red;");
    }

    #[test]
    fn style_fold_keeps_list_order_so_the_later_list_overrides() {
        let a = merge_attributes(vec![
            vec![attr("style", "color: red;")],
            vec![attr("style", "anchor-name: --dxa-x;")],
        ]);
        assert_eq!(get_value(&a[0]), "color: red; anchor-name: --dxa-x;");
        // A later declaration of the same property comes last, so it wins the cascade.
        let b = merge_attributes(vec![
            vec![attr("style", "anchor-name: --a;")],
            vec![attr("style", "anchor-name: --b;")],
        ]);
        assert_eq!(get_value(&b[0]), "anchor-name: --a; anchor-name: --b;");
    }

    /// Dioxus appends a component call's ad-hoc `style:` key after its explicit `attributes`
    /// field, so both can arrive in ONE list (`Button { style: "..", attributes }`).
    #[test]
    fn two_styles_inside_one_list_are_folded() {
        let result = merge_attributes(vec![
            vec![attr("class", "dx-button")],
            vec![
                attr("id", "t"),
                attr("style", "anchor-name: --dxa-x;"),
                attr("style", "border-radius: 9999px;"),
            ],
        ]);
        assert_eq!(result.iter().filter(|a| a.name == "style").count(), 1);
        let style = result.iter().find(|a| a.name == "style").unwrap();
        assert_eq!(
            get_value(style),
            "anchor-name: --dxa-x; border-radius: 9999px;"
        );
    }

    #[test]
    fn style_fold_supplies_missing_semicolon_and_trims() {
        let result = merge_attributes(vec![
            vec![attr("style", "  anchor-name: --dxa-x  ")],
            vec![attr("style", "  color: red  ")],
        ]);
        assert_eq!(get_value(&result[0]), "anchor-name: --dxa-x; color: red");
        // The anchor declaration keeps the trailing `;` that
        // `[style*="anchor-name: --dxa-{id};"]` selectors look for.
        assert!(get_value(&result[0]).contains("anchor-name: --dxa-x;"));
    }

    #[test]
    fn style_fold_with_empty_side_returns_the_other_side() {
        let a = merge_attributes(vec![
            vec![attr("style", "")],
            vec![attr("style", "color: red;")],
        ]);
        assert_eq!(get_value(&a[0]), "color: red;");
        let b = merge_attributes(vec![
            vec![attr("style", "color: red;")],
            vec![attr("style", " ")],
        ]);
        assert_eq!(get_value(&b[0]), "color: red;");
        let c = merge_attributes(vec![vec![attr("style", "")], vec![attr("style", "")]]);
        assert_eq!(c.len(), 1);
        assert_eq!(get_value(&c[0]), "");
    }

    #[test]
    fn style_fold_ignores_a_later_none_and_adopts_a_later_value_over_none() {
        let none = |name: &'static str| Attribute {
            name,
            namespace: None,
            volatile: false,
            value: dioxus_core::AttributeValue::None,
        };
        let kept = merge_attributes(vec![
            vec![attr("style", "anchor-name: --dxa-x;")],
            vec![none("style")],
        ]);
        assert_eq!(kept.len(), 1);
        assert_eq!(get_value(&kept[0]), "anchor-name: --dxa-x;");
        let adopted = merge_attributes(vec![
            vec![none("style")],
            vec![attr("style", "color: red;")],
        ]);
        assert_eq!(adopted.len(), 1);
        assert_eq!(get_value(&adopted[0]), "color: red;");
    }

    #[test]
    fn style_fold_ors_the_volatile_flag() {
        let mut a1 = attr("style", "a: b;");
        a1.volatile = true;
        let result = merge_attributes(vec![vec![a1], vec![attr("style", "c: d;")]]);
        assert!(result[0].volatile);
    }

    /// The duplicate-attribute protections are unchanged: still exactly one attribute per
    /// `(name, namespace)`, the shorthand `style` namespace is NOT folded into the plain
    /// `style` (that is `fold_style_attributes`'s job), and every non-`class`/`style` name is
    /// still overwritten by the last occurrence.
    #[test]
    fn duplicate_protections_still_hold() {
        let shorthand = Attribute {
            name: "padding",
            namespace: Some("style"),
            volatile: false,
            value: Text("1rem".into()),
        };
        let result = merge_attributes(vec![
            vec![attr("style", "a: b;"), attr("id", "one"), attr("role", "x")],
            vec![attr("style", "c: d;"), attr("id", "two"), shorthand.clone()],
            vec![attr("style", "e: f;"), attr("id", "three")],
        ]);
        let mut keys: Vec<_> = result.iter().map(|a| (a.name, a.namespace)).collect();
        let total = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), total, "a (name, namespace) pair appeared twice");
        let get = |n: &str| result.iter().find(|a| a.name == n).map(get_value);
        assert_eq!(get("style"), Some("a: b; c: d; e: f;"));
        assert_eq!(get("id"), Some("three"));
        assert_eq!(get("padding"), Some("1rem"));
        assert_eq!(get("role"), Some("x"));
    }

    /// `fold_style_attributes` is the other half (shorthand + plain into one string) and is
    /// untouched by `merge_attributes` folding `style`.
    #[test]
    fn fold_style_attributes_still_collects_plain_and_shorthand() {
        let shorthand = Attribute {
            name: "padding",
            namespace: Some("style"),
            volatile: false,
            value: Text("1rem".into()),
        };
        let (style, rest) = fold_style_attributes(vec![
            attr("style", "color: red"),
            shorthand,
            attr("id", "x"),
        ]);
        assert_eq!(style.as_deref(), Some("color: red;padding:1rem;"));
        assert_eq!(rest.len(), 1);
    }

    #[test]
    fn class_attributes_are_merged() {
        let result = merge_attributes(vec![vec![attr("class", "foo")], vec![attr("class", "bar")]]);
        assert_eq!(result.len(), 1);
        assert_eq!(get_value(&result[0]), "foo bar");
    }

    #[test]
    fn class_merge_trims_whitespace() {
        let result = merge_attributes(vec![
            vec![attr("class", "  foo  ")],
            vec![attr("class", "  bar  ")],
        ]);
        assert_eq!(get_value(&result[0]), "foo bar");
    }

    #[test]
    fn class_merge_handles_empty() {
        let result = merge_attributes(vec![vec![attr("class", "")], vec![attr("class", "bar")]]);
        assert_eq!(get_value(&result[0]), "bar");
    }

    #[test]
    fn mixed_attributes() {
        let result = merge_attributes(vec![
            vec![attr("class", "a"), attr("id", "x")],
            vec![attr("class", "b"), attr("id", "y")],
        ]);
        assert_eq!(result.len(), 2);
        // Should be sorted by name
        assert_eq!(result[0].name, "class");
        assert_eq!(result[1].name, "id");
        // class merged, id overwritten
        assert_eq!(get_value(&result[0]), "a b");
        assert_eq!(get_value(&result[1]), "y");
    }

    #[test]
    fn unsorted_input_still_works() {
        // Even if inputs aren't sorted, the function should handle it
        let result = merge_attributes(vec![
            vec![attr("z", "1"), attr("a", "2")],
            vec![attr("m", "3")],
        ]);
        assert_eq!(result.len(), 3);
        // Output should be sorted
        assert_eq!(result[0].name, "a");
        assert_eq!(result[1].name, "m");
        assert_eq!(result[2].name, "z");
    }

    #[test]
    fn volatile_flag_preserved_on_class_merge() {
        let mut a1 = attr("class", "foo");
        a1.volatile = true;
        let a2 = attr("class", "bar");

        let result = merge_attributes(vec![vec![a1], vec![a2]]);
        assert!(result[0].volatile);
    }

    // ---- event listeners: same-named listeners chain instead of replacing ----

    type Log = Rc<RefCell<Vec<&'static str>>>;

    fn listener_attr(name: &'static str, log: &Log, tag: &'static str) -> Attribute {
        let log = log.clone();
        Attribute {
            name,
            namespace: None,
            volatile: false,
            value: dioxus_core::AttributeValue::listener(
                move |_: dioxus_core::Event<dioxus::html::PlatformEventData>| {
                    log.borrow_mut().push(tag);
                },
            ),
        }
    }

    /// Like [`listener_attr`], but the listener calls `prevent_default()` first.
    fn preventing_listener_attr(name: &'static str, log: &Log, tag: &'static str) -> Attribute {
        let log = log.clone();
        Attribute {
            name,
            namespace: None,
            volatile: false,
            value: dioxus_core::AttributeValue::listener(
                move |event: dioxus_core::Event<dioxus::html::PlatformEventData>| {
                    event.prevent_default();
                    log.borrow_mut().push(tag);
                },
            ),
        }
    }

    /// Fire the `name` listener of `attrs` the way a renderer does (an
    /// `Event<PlatformEventData>` erased to `dyn Any`).
    fn fire(attrs: &[Attribute], name: &str) {
        let attr = attrs.iter().find(|a| a.name == name).expect("listener");
        let dioxus_core::AttributeValue::Listener(l) = &attr.value else {
            panic!("expected a listener")
        };
        let data = dioxus::html::PlatformEventData::new(Box::new(()));
        l.call(Event::new(Rc::new(data), true).into_any());
    }

    /// Listeners need a runtime (`ListenerCallback::new` reads the current scope).
    fn in_runtime(f: impl Fn() + Clone + 'static) {
        #[derive(Clone, Props)]
        struct P {
            f: Rc<dyn Fn()>,
        }
        impl PartialEq for P {
            fn eq(&self, _: &Self) -> bool {
                true
            }
        }
        fn app(p: P) -> Element {
            (p.f)();
            rsx! {}
        }
        let mut dom = VirtualDom::new_with_props(app, P { f: Rc::new(f) });
        dom.rebuild_in_place();
    }

    #[test]
    fn same_named_listeners_both_run_callers_first() {
        in_runtime(|| {
            let log: Log = Default::default();
            let merged = merge_attributes(vec![
                vec![listener_attr("onclick", &log, "primitive")],
                vec![listener_attr("onclick", &log, "caller")],
            ]);
            assert_eq!(merged.len(), 1);
            fire(&merged, "onclick");
            assert_eq!(*log.borrow(), ["caller", "primitive"]);
        });
    }

    /// Radix `composeEventHandlers`: the caller's listener preventing default skips the
    /// component's own; one that does not prevent leaves it running.
    #[test]
    fn caller_prevent_default_skips_the_components_own_listener() {
        in_runtime(|| {
            let log: Log = Default::default();
            let merged = merge_attributes(vec![
                vec![listener_attr("onclick", &log, "primitive")],
                vec![preventing_listener_attr("onclick", &log, "caller")],
            ]);
            fire(&merged, "onclick");
            assert_eq!(*log.borrow(), ["caller"]);
        });
    }

    #[test]
    fn prevent_default_in_a_middle_list_skips_only_the_lists_before_it() {
        in_runtime(|| {
            let log: Log = Default::default();
            let merged = merge_attributes(vec![
                vec![listener_attr("onclick", &log, "a")],
                vec![preventing_listener_attr("onclick", &log, "b")],
                vec![listener_attr("onclick", &log, "c")],
            ]);
            fire(&merged, "onclick");
            assert_eq!(*log.borrow(), ["c", "b"]);
        });
    }

    #[test]
    fn three_lists_run_last_list_first() {
        in_runtime(|| {
            let log: Log = Default::default();
            let merged = merge_attributes(vec![
                vec![listener_attr("onclick", &log, "a")],
                vec![listener_attr("onclick", &log, "b")],
                vec![listener_attr("onclick", &log, "c")],
            ]);
            fire(&merged, "onclick");
            assert_eq!(*log.borrow(), ["c", "b", "a"]);
        });
    }

    /// A button whose own (prop) handlers ride last and a caller listener in the spread: the
    /// shape of `Button` inside `DialogClose` (row 155 groups 4 and 6).
    #[test]
    fn caller_spread_listener_survives_a_later_handlers_list() {
        in_runtime(|| {
            let log: Log = Default::default();
            let from_spread = vec![listener_attr("onclick", &log, "spread")];
            let own = vec![
                listener_attr("onclick", &log, "prop"),
                listener_attr("onkeydown", &log, "key"),
            ];
            let merged = merge_attributes(vec![vec![attr("class", "b")], own, from_spread]);
            fire(&merged, "onclick");
            assert_eq!(*log.borrow(), ["spread", "prop"]);
            fire(&merged, "onkeydown");
            assert_eq!(*log.borrow(), ["spread", "prop", "key"]);
        });
    }

    #[test]
    fn the_same_callback_in_two_lists_runs_once() {
        in_runtime(|| {
            let log: Log = Default::default();
            let one = listener_attr("onclick", &log, "once");
            let merged = merge_attributes(vec![vec![one.clone()], vec![one]]);
            fire(&merged, "onclick");
            assert_eq!(*log.borrow(), ["once"]);
        });
    }

    #[test]
    fn different_events_do_not_chain_and_none_still_removes() {
        in_runtime(|| {
            let log: Log = Default::default();
            let merged = merge_attributes(vec![
                vec![listener_attr("onclick", &log, "click")],
                vec![listener_attr("onkeydown", &log, "key")],
            ]);
            fire(&merged, "onkeydown");
            assert_eq!(*log.borrow(), ["key"]);

            let removed = merge_attributes(vec![
                vec![listener_attr("onclick", &log, "click")],
                vec![Attribute {
                    name: "onclick",
                    namespace: None,
                    volatile: false,
                    value: dioxus_core::AttributeValue::None,
                }],
            ]);
            assert!(matches!(
                removed[0].value,
                dioxus_core::AttributeValue::None
            ));
        });
    }
}

#[cfg(test)]
mod unique_id_tests {
    use super::format_unique_id;

    #[test]
    fn server_ids_keep_the_plain_shape() {
        assert_eq!(format_unique_id(true, 0), "dxc-0");
        assert_eq!(format_unique_id(true, 229), "dxc-229");
    }

    #[test]
    fn client_ids_use_a_disjoint_prefix() {
        assert_eq!(format_unique_id(false, 0), "dxc-c0");
        assert_eq!(format_unique_id(false, 229), "dxc-c229");
    }

    /// The regression: a client-allocated id equal in counter value to an SSR
    /// id (`dxc-229` vs the client's own 229th) must still differ.
    #[test]
    fn client_and_server_ids_never_collide_for_any_counter_values() {
        for server_n in 0..400 {
            for client_n in 0..400 {
                assert_ne!(
                    format_unique_id(true, server_n),
                    format_unique_id(false, client_n),
                    "server {server_n} vs client {client_n}"
                );
            }
        }
    }

    /// Neither id is a substring of a different id of the same side with the
    /// trailing `;` that `anchor-name: --dxa-{id};` selectors rely on (see
    /// `top_layer::use_anchor_position_fallback`).
    #[test]
    fn terminated_ids_are_not_substrings_of_each_other() {
        let ids: Vec<String> = (0..120)
            .flat_map(|n| [format_unique_id(true, n), format_unique_id(false, n)])
            .collect();
        for a in &ids {
            for b in &ids {
                if a != b {
                    assert!(!format!("{b};").contains(&format!("{a};")), "{a} in {b}");
                }
            }
        }
    }
}
