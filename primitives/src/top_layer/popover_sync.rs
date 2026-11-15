//! `PopoverKind` and the popover attribute <-> signal sync hooks.

#[cfg_attr(
    feature = "native",
    expect(unused_imports, reason = "only the browser build uses it")
)]
use super::*;

/// Which `popover` dismissal behaviour an element declares.
/// WHATWG HTML §the-popover-attribute (see module docs for the link).
///
/// Only referenced from the `#[cfg(not(feature = "native"))]` leaf render
/// functions in `tooltip.rs`/`hover_card.rs`/`popover.rs` -- the native
/// (Blitz) arm never renders the `popover` attribute at all (see
/// `use_popover_sync`'s doc) -- so this whole type is `#[cfg]`-gated too,
/// rather than degrading to a dead stub on that target.
#[cfg(not(feature = "native"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PopoverKind {
    /// `popover="auto"` — light-dismisses on Escape and on a pointerdown
    /// outside the popover (and any popovers it is nested in); showing one
    /// auto popover also closes unrelated open ones. Used by the non-modal
    /// [`crate::popover`] arm, which needs no dismiss-core layer that closes it on the browser arm
    /// (`use_dismiss`, `dismiss.rs`): the browser's light dismiss does that, and the arm's `NATIVE`
    /// layer only keeps its place in the Escape order;
    /// also `dropdown_menu.rs`, `menubar.rs`, and `select/components/
    /// list.rs`'s `SelectListRendered`, each of
    /// which layers light dismiss on top of -- not instead of -- their own
    /// existing blur-driven dismissal as a backstop; see each's own doc for
    /// why `auto` is safe there specifically.
    ///
    /// **Opening-gesture contract (dev-docs/backlog.md row 40):** the browser's
    /// light dismiss reads only trusted `pointerdown`/`pointerup`, so an
    /// `auto` popover that becomes open *between* a gesture's `pointerdown`
    /// and `pointerup` is hidden by that same gesture's `pointerup`. An
    /// `auto` popover must therefore only be shown from `click`,
    /// `pointerup`, keyboard, hover-compat or timer handlers -- never from
    /// `onpointerdown`. Pinned by
    /// `playwright/oracle/tier2-html/auto-popover-opening-gesture.spec.ts`.
    Auto,
    /// `popover="manual"` — no light-dismiss; only an explicit
    /// `hidePopover()` (here, driven by our own `open` signal going false)
    /// closes it. Used by [`crate::tooltip`] and [`crate::hover_card`],
    /// which already own their entire open/close lifecycle via hover/focus
    /// and gain nothing from -- and would fight with -- native light
    /// dismiss (a `HoverCardContent` mouseenter re-opens across what would
    /// otherwise be an outside click if `auto` briefly closed it first);
    /// also `context_menu.rs` (a point-opened menu with no persistent
    /// trigger for `auto` to reason about) and `combobox/components/
    /// list.rs`'s `ComboboxListRendered` (`auto` fails there -- see that
    /// component's doc for the concrete failure).
    Manual,
}

#[cfg(not(feature = "native"))]
impl PopoverKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Manual => "manual",
        }
    }
}

/// Drives the open level of the `[popover]` element `id` from `open` through the page executor
/// (`bridge::executor`, row 449): `showPopover()` when `open` turns true, `hidePopover()` when it turns false, and
/// a level the page reaches on its own (light dismiss, Escape) back into `set_open`, so the Rust signal never
/// strands. The executor is the element's one writer: its own show and hide are never echoed back, and a report
/// the owner has decided over since is dropped (`bridge::executor`'s module docs), so a late `toggle` cannot reopen a
/// menu that was just closed.
///
/// For a panel that is always mounted (the menu family, through `use_anchored_panel`'s `PanelSync::Immediate`).
///
/// # Renderer-support note
///
/// Only a real browser or webview implements the Popover API. Blitz (`dioxus-native`) implements neither the
/// `popover` attribute nor `document::eval` (`dev-docs/reference/platform-rationale.md` Caveat 2), so this hook and
/// the `popover` attribute its callers render exist only on the `not(feature = "native")` arm: Blitz's UA sheet
/// hides `[popover]:not(:popover-open)` without implementing `:popover-open`, so the attribute would hide the
/// element for good there. The gate is the renderer feature, not `target_family = "wasm"`: `dioxus-desktop` is a
/// non-wasm binary with a working webview, and the fullstack SSG prerender is a host binary that renders the
/// browser arm's markup (Caveat 1).
#[cfg(not(feature = "native"))]
pub(crate) fn use_popover_sync(
    id: String,
    open: impl Readable<Target = bool> + Copy + 'static,
    set_open: Callback<bool>,
) {
    // The ink baseline (`ensure_top_layer_ink_styles`) is evaluated before the executor's session is opened: both
    // are evals created in this order, and the page runs evals in the order they are created.
    use_effect(ensure_top_layer_ink_styles);
    crate::bridge::executor::use_popover_level(
        id,
        open,
        crate::bridge::executor::ShowMode::Immediate,
        set_open,
        crate::site!("executor.popover"),
    );
}

/// [`use_popover_sync`] for content whose own `data-open` / `data-closed` animates through a "closed but still
/// mounted" phase (`use_animated_open`): `showPopover()` runs every time `open` turns true, and a closed level hides
/// nothing (`ShowMode::Mounted`), so only the owner's unmount takes the content out of the top layer.
///
/// **The exit animation.** `hidePopover()` on the closing path would set `display: none` (the UA's
/// `[popover]:not(:popover-open)`) on the tick `open` turns false, before `use_animated_open`'s rAF-deferred
/// `getAnimations()` read, so the exit would be skipped (`combobox.spec.ts`, "keeps filtered options during
/// keyboard close animation").
///
/// **A reopen while still mounted.** An `auto` popover's light dismiss treats the trigger's own `pointerdown` as
/// outside (`SelectTrigger` and `ComboboxInput` are not its invoker), so a click on the trigger of a closing,
/// still-shown list hides it for real and the trigger's `onclick` reopens it in the same gesture. The show is
/// applied on every open level, whatever ended the previous one (`combobox.spec.ts`, "filters and selects with the
/// keyboard").
///
/// Only closes reach `on_native_close`: the executor shows the element only on the owner's command, so the page
/// has no open of its own to report. A native close (Escape, a press outside, `auto` only) skips the exit
/// animation, the accepted limit of the Popover API without `@starting-style`.
#[cfg(not(feature = "native"))]
pub(crate) fn use_popover_shown_while_mounted(
    id: String,
    open: impl Readable<Target = bool> + Copy + 'static,
    on_native_close: Callback<bool>,
) {
    use_effect(ensure_top_layer_ink_styles);
    let on_page = use_callback(move |is_open: bool| {
        if !is_open {
            on_native_close.call(false);
        }
    });
    crate::bridge::executor::use_popover_level(
        id,
        open,
        crate::bridge::executor::ShowMode::Mounted,
        on_page,
        crate::site!("executor.popover"),
    );
}

/// [`use_popover_shown_while_mounted`] for a `<dialog popover>` panel (the non-modal Popover, so DatePicker and
/// ColorPicker), whose show runs the open-focus policy (`dxOpenFocus`, `modality.rs`): where focus lands follows
/// the opening modality like a modal's does. The policy is a legacy page script until its decisions are Rust's
/// (row 456), so this panel keeps the legacy show script and close listener (both ported with it).
#[cfg(not(feature = "native"))]
pub(crate) fn use_dialog_panel_shown_while_mounted(
    id: String,
    open: impl Readable<Target = bool> + Copy + 'static,
    on_native_close: Callback<bool>,
) {
    // Installs `ensure_top_layer_ink_styles`'s engine-injected ink baseline: see that function's doc.
    use_effect(ensure_top_layer_ink_styles);

    // Browser -> signal: forwards every native *close* `toggle` for this
    // element's whole mounted lifetime -- and ONLY closes (dev-docs/backlog.md
    // row 115). `toggle` is dispatched asynchronously (a queued task), so
    // the `toggle(newState: "open")` echo of this hook's own `showPopover()`
    // call can arrive AFTER the signal has already gone `false` again (a
    // focus-out/mouse-leave in the same beat as the open: input events
    // outrank that queued task under main-thread load). Mirroring that
    // stale "open" back would flip `open` to `true` with nothing left to
    // close it -- a tooltip/hover card stuck open forever (calling `blur()`
    // right after `showPopover()` triggers it deterministically).
    // The `open` direction is never needed: this hook's effect below is the
    // only `showPopover()` caller for the element (the signal always opens
    // it, never the browser), so a native "open" can only be an echo of our
    // own call. Native closes (light dismiss, Escape) have no such echo --
    // the browser acted first -- and still sync back.
    let id_for_listener = id.clone();
    crate::js_listener::use_js_listeners(move || {
        Some(
            crate::js_listener::JsListeners::install(
                "const id = await dioxus.recv();
                listen(document.getElementById(id), 'toggle', (e) => { if (e.newState !== 'open') dioxus.send(false); });",
            )
            .arg(id_for_listener.clone())
            .on_message(move |is_open: bool| on_native_close.call(is_open)),
        )
    });

    // Signal -> browser, show-only: `showPopover()` runs every time `open` is true and the browser does not
    // already agree (the reopen while still mounted, `use_popover_shown_while_mounted`'s docs); `hidePopover()` is
    // never called here (the exit animation, same docs), so only a real unmount closes this content.
    use_effect(move || {
        if !open.cloned() {
            return;
        }
        let id = id.clone();
        // Prepended into this exact script rather than left to the separate `use_effect(
        // ensure_top_layer_ink_styles)` above: `ensure_top_layer_ink_styles`'s doc, "First-paint window".
        let inject = TOP_LAYER_INK_STYLES_INJECT_JS;
        let modality = crate::modality::MODALITY_JS;
        let open_focus = crate::modality::OPEN_FOCUS_JS;
        let id = crate::escape::js_string(&id);
        #[expect(
            clippy::disallowed_methods,
            reason = "row 452: this page script is ported by row 452"
        )]
        document::eval(&format!(
            "{inject}
            {modality};
            {open_focus}
            const el = document.getElementById({id});
            if (!el || el.matches(':popover-open')) return;
            // A `<dialog popover>` is a dialog panel (the non-modal Popover, so DatePicker and
            // ColorPicker): where focus lands follows the opening modality like a modal's does
            // (`dxOpenFocus`, `modality.rs`). Every other popover (tooltip, hover card, menus,
            // select list) keeps the browser's own focusing step, or none.
            if (el.tagName === 'DIALOG') dxOpenFocus(el, () => el.showPopover());
            else el.showPopover();"
        ));
    });
}
