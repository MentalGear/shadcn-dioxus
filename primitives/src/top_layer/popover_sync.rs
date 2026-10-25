//! `PopoverKind` and the popover attribute <-> signal sync hooks.

#[cfg_attr(not(feature = "web"), allow(unused_imports))]
use super::*;

/// Which `popover` dismissal behaviour an element declares.
/// WHATWG HTML §the-popover-attribute (see module docs for the link).
///
/// Only referenced from the `#[cfg(feature = "web")]` leaf render
/// functions in `tooltip.rs`/`hover_card.rs`/`popover.rs` -- the native
/// (Blitz) arm never renders the `popover` attribute at all (see
/// `use_popover_sync`'s doc) -- so this whole type is `#[cfg]`-gated too,
/// rather than degrading to a dead stub on that target.
#[cfg(feature = "web")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PopoverKind {
    /// `popover="auto"` — light-dismisses on Escape and on a pointerdown
    /// outside the popover (and any popovers it is nested in); showing one
    /// auto popover also closes unrelated open ones. Used by the non-modal
    /// [`crate::popover`] arm, which needs no `use_outside_dismiss`/
    /// `use_global_escape_listener` wiring on the web arm;
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

#[cfg(feature = "web")]
impl PopoverKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Manual => "manual",
        }
    }
}

/// Drive `showPopover()`/`hidePopover()` on the element with the given `id`
/// from `open`, and sync the browser's own `toggle` event — fired on
/// light-dismiss (`auto`), Escape (`auto`), or any other script/UA-driven
/// close — back into `set_open`, so the Rust signal never strands. This is
/// the same defect class `dev-docs/reference/platform-rationale.md` Caveat 1
/// documents for `<dialog>`'s one-way `showModal()`/`close()` binding;
/// the eval-channel
/// pattern that fixes it here is the same one already used by
/// `use_form_reset_listener` (`lib.rs`).
///
/// The show/hide effect checks `:popover-open` before calling either
/// method, mirroring the native-`<dialog>` spike's `dialog.open` guard
/// (`dev-docs/archive/research/phase4-spike-findings.md`, "An `.open` guard") — without it, a
/// redundant call (the signal already agrees with the browser's state)
/// throws `InvalidStateError`.
///
/// # Renderer-support note
///
/// Only a real browser or webview implements the Popover API. Blitz
/// (`dioxus-native`) implements neither the `popover` attribute nor
/// `document::eval` (the latter is a no-op there) —
/// `dev-docs/reference/platform-rationale.md` Caveat 2's capability table shows
/// both at zero. This hook must never run on that target: calling it there
/// would be a silent no-op at best (`eval` swallowed) and, worse, its
/// *caller* rendering the `popover` attribute unconditionally would be
/// actively harmful — Blitz's UA stylesheet borrows Firefox's
/// `[popover]:not(:popover-open) { display: none }` rule without
/// implementing `:popover-open` itself, so any element carrying the
/// attribute there would always match `:not(:popover-open)` and render
/// permanently hidden. See each call site (`tooltip.rs`, `hover_card.rs`,
/// `popover.rs`) for the matching `#[cfg]` split on the attribute itself.
///
/// It is gated on `feature = "web"` (a renderer Cargo feature this crate
/// already defines), not `target_family = "wasm"` — `dioxus-desktop` is a
/// non-wasm binary with a real, working webview `eval`, and belongs on this
/// same web-style arm, not on the native/no-op one `target_family = "wasm"`
/// would wrongly exclude it from. A `target_family = "wasm"` gate would
/// also have the fullstack SSG prerender — a host (non-wasm) binary built with
/// this feature on — render the native-arm markup instead
/// (`dev-docs/reference/platform-rationale.md` Caveat 1). See that caveat
/// for the rule: markup/attribute choice keys off this renderer
/// feature; only genuinely wasm-only *execution* internals (none exist in
/// this hook's body — `document::eval` itself compiles and runs inertly on
/// any renderer with no document context) would still key off
/// `target_family`.
#[cfg(feature = "web")]
pub(crate) fn use_popover_sync(
    id: String,
    open: impl Readable<Target = bool> + Copy + 'static,
    set_open: Callback<bool>,
) {
    // Installs `ensure_top_layer_ink_styles`'s engine-injected ink baseline
    // -- see that function's own doc for what it fixes and why every
    // `use_popover_sync` call site (every plain-lifecycle `[popover]`
    // consumer) is one of the three provably-exhaustive call sites it lists.
    use_effect(ensure_top_layer_ink_styles);

    // Browser -> signal.
    let id_for_listener = id.clone();
    crate::js_listener::use_js_listeners(move || {
        Some(
            crate::js_listener::JsListeners::install(
                "const id = await dioxus.recv();
                listen(document.getElementById(id), 'toggle', (e) => dioxus.send(e.newState === 'open'));",
            )
            .arg(id_for_listener.clone())
            .on_message(move |is_open: bool| set_open.call(is_open)),
        )
    });

    // Signal -> browser.
    use_effect(move || {
        let want_open = open.cloned();
        let id = id.clone();
        // `inject` is prepended into this exact script, not left to the
        // separate `use_effect(ensure_top_layer_ink_styles)` above --
        // see `ensure_top_layer_ink_styles`'s doc, "First-paint window,"
        // for why relying on that other effect's `document::eval()` having
        // already run by the time *this* one calls `showPopover()` is not
        // safe to assume, and why prepending (one synchronous script, not
        // two separately-ordered ones) is what actually closes the gap.
        let inject = TOP_LAYER_INK_STYLES_INJECT_JS;
        document::eval(&format!(
            "{inject}
            const el = document.getElementById('{id}');
            if (!el) return;
            const isOpen = el.matches(':popover-open');
            if ({want_open} && !isOpen) el.showPopover();
            if (!{want_open} && isOpen) el.hidePopover();"
        ));
    });
}

/// Variant of [`use_popover_sync`] for content whose *own* `data-state`
/// (Radix-style) animates through a "closed but still mounted" phase --
/// `use_animated_open`-driven content, unlike every other
/// `use_popover_sync` call site. Forwards native/browser closes to
/// `on_native_close` exactly like [`use_popover_sync`]; the difference is
/// entirely on the "signal -> browser" side: `showPopover()` still runs
/// reactively, every time `open` becomes `true` (not once, at mount -- see
/// "Why this exists" below for why once is not enough), but
/// **`hidePopover()` is never called on our own closing path at all** --
/// only a real DOM removal (this component's caller unmounting it once its
/// own exit animation settles) ever takes this content out of the top
/// layer.
///
/// ## Why this exists: two bugs the plain sync hook cannot avoid
///
/// [`use_popover_sync`]'s own "signal -> browser" effect calls
/// `hidePopover()` the instant `open` goes `false`. That is exactly right
/// for content with no exit animation (`Tooltip`/`HoverCard`/`Popover`/
/// `DropdownMenu`/`ContextMenu`/`Menubar`, none of which keep their content
/// mounted through a CSS animate-out -- Radix-style `data-state="closed"`
/// animations on those are cosmetic fades that finish well after the
/// element would already be gone). `SelectList`/`ComboboxList` are
/// different: both render through `use_animated_open`, which
/// deliberately keeps the content mounted with `data-state="closed"` for
/// its whole exit animation (plus a settle hold) before actually
/// unmounting it -- `combobox.spec.ts`'s "keeps filtered options during
/// keyboard close animation" test asserts exactly this, that the
/// `data-state="closed"` element is still visible mid-animation.
///
/// **Bug 1 (animation race).** Calling [`use_popover_sync`] directly with the
/// raw `open` signal fails that test: `hidePopover()` fires on the same
/// tick `open` goes `false`, which (per the UA popover stylesheet's
/// `[popover]:not(:popover-open) { display: none }`) sets `display: none`
/// on the content *before* `use_animated_open`'s own rAF-deferred
/// `getAnimations()` check ever runs, so it observes zero running
/// animations and finishes the close cycle immediately -- the exit
/// animation is skipped outright, and Playwright's `toBeVisible()` on the
/// still-should-be-animating `data-state="closed"` element fails. Never
/// calling `hidePopover()` from the closing path at all -- only from a real
/// unmount, once `use_animated_open` itself has already decided the
/// animation (and its settle hold) is done -- sidesteps this entirely: the
/// exit animation plays out undisturbed on an element that is still very
/// much `:popover-open` the whole time.
///
/// **Bug 2 (reopen-while-still-mounted, `auto` only).** Covered by
/// `combobox.spec.ts`'s "filters and selects with the keyboard" test
/// (it would go red intermittently were this not handled):
/// `auto`'s native light dismiss has no notion of "this input is the
/// trigger that owns me" (`ComboboxInput`/`SelectTrigger` are never
/// declared as this popover's invoker, and neither a `popovertarget`
/// attribute nor `preventDefault()` on the
/// trigger's own `pointerdown`/`click` stops it; WHATWG light dismiss
/// treats *any* pointerdown outside the popover's own DOM subtree as
/// "outside," full stop). So: select an option (closes, `open` -> `false`,
/// content stays mounted mid-exit-animation per Bug 1 above), then
/// -- while that same instance is *still* mounted, `:popover-open` still
/// `true` -- click the trigger again to reopen. The trigger's own
/// `pointerdown` is classified as "outside" by native light dismiss, which
/// closes the *still-showing* popover for real (a genuine
/// `showPopover`/`hidePopover` pair, not merely a `data-state` flip) --
/// forwarded correctly back into `on_native_close`, setting `open` to
/// `false` -- **immediately followed**, same gesture, by the trigger's own
/// `onclick` reading `open` and calling `set_open(true)` to reopen it. A
/// one-time, mount-only `showPopover()` call never re-fires for this
/// already-mounted instance, so `open` becomes `true` again in
/// Rust/`data-state` while the actual DOM element stays hidden
/// (`:not(:popover-open)`) -- a user-visible "second click does nothing"
/// defect. Reacting to `open` on every transition (not just the first)
/// avoids it the same way [`use_popover_sync`]'s own show effect does:
/// whenever `open` is `true` and the element is not yet `:popover-open`,
/// `showPopover()` runs again, regardless of *why* it had stopped being
/// shown.
///
/// A *native* close (Escape / outside pointerdown, `auto` only) still
/// bypasses the exit animation on its own path -- the browser's own hide
/// algorithm sets `display: none` synchronously, before Rust ever learns
/// about it -- but that is an accepted, well-known limit of the plain
/// Popover API without `@starting-style`/`transition-behavior:
/// allow-discrete` (out of scope here), not something the oracle
/// requires animated; the test Bug 1 targets only exercises a
/// script-driven (Enter-key) close, exactly the path this hook keeps
/// animatable.
#[cfg(feature = "web")]
pub(crate) fn use_popover_shown_while_mounted(
    id: String,
    open: impl Readable<Target = bool> + Copy + 'static,
    on_native_close: Callback<bool>,
) {
    // Installs `ensure_top_layer_ink_styles`'s engine-injected ink baseline
    // -- see that function's own doc for what it fixes and why every
    // `use_popover_shown_while_mounted` call site (every animated-exit
    // `[popover]` consumer) is one of the three provably-exhaustive call
    // sites it lists.
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

    // Signal -> browser, show-only: reacts to the real `open` signal (not a
    // one-shot mount effect -- see "Bug 2" above for why a reopen needs
    // this to run again) so `showPopover()` fires every time `open` is
    // `true` and the browser doesn't already agree; `hidePopover()` is
    // deliberately never called here at all -- see "Bug 1" above -- only a
    // real unmount ever closes this content.
    use_effect(move || {
        if !open.cloned() {
            return;
        }
        let id = id.clone();
        // See `use_popover_sync`'s matching comment / `ensure_top_layer_
        // ink_styles`'s doc, "First-paint window": prepended into this
        // exact script rather than left to the separate `use_effect(
        // ensure_top_layer_ink_styles)` above, for the same reason.
        let inject = TOP_LAYER_INK_STYLES_INJECT_JS;
        let modality = crate::modality::MODALITY_JS;
        let open_focus = crate::modality::OPEN_FOCUS_JS;
        document::eval(&format!(
            "{inject}
            {modality};
            {open_focus}
            const el = document.getElementById('{id}');
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
