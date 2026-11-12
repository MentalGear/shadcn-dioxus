---
id: 511
title: "The primitives' web feature is the browser-engine switch and nothing states or enforces it: a CLI-installed app silently gets the Blitz arms"
status: "OPEN, found 2026-11-12. Needs the owner's choice between options A, B and C before any code. Found…"
size: "M (inferred); A is S, C is S to M, B is…"
area: primitives
updated: 2026-11-12
---

# Row 511: the primitives' `web` feature is the browser-engine switch, and nothing states or enforces it

## Item

The primitives' `web` Cargo feature means "a browser engine renders this" (`scripts/check-cfg-axis.sh:5-7`; not "this is wasm": it is true for the wasm client, the desktop and mobile webview and the SSG server build alike). The library's whole renderer split hangs on it: every `#[cfg(feature = "web")]` / `#[cfg(not(feature = "web"))]` pair under `primitives/src/` chooses between the browser arm and the Blitz arm. Nothing states or enforces that an app turns it on. This repository's own apps do (`preview/Cargo.toml:38-41` for `web`, `desktop` and `server`; `templates/starter/Cargo.toml.liquid:17-20` for `web`, `desktop` and `mobile`). The install path does not: the CLI adds the primitives dependency as `cargo add --no-default-features` plus each component's own features (`cli/src/manifest.rs:110-115`) and never names `web`, and `scripts/registry-install-smoke.sh:157-167` strips `dioxus-primitives/web` from the app and builds without it. So an app that installs components with the CLI and is not made from the starter template silently gets the Blitz arms on every renderer, the web and the webview included.

What a user sees differently between the two arms (read from the source, not observed in a browser):

- **Modal dialog and alert dialog.** `web`: a real `<dialog>` opened with `showModal()` (`dialog.rs:568`, `alert_dialog.rs:315`), so the browser supplies the focus trap, focus restore, background inertness and top layer. `not(web)`: a plain `div` with `role="dialog"` and `aria-modal` (`dialog.rs:480`, `alert_dialog.rs:233`), no top layer (an ancestor with `overflow: hidden` or a transform clips it), no inertness for the background, and the focus trap is the bundled `focus-trap.js` driven by a `document::eval` (`dialog.rs:521`, `alert_dialog.rs:264`).
- **Popover.** `web`: `popover="auto"` or `"manual"` with the top-layer helpers (`popover.rs:592`, `:735`). `not(web)`: a plain in-flow `div` with listener-driven dismissal (`popover.rs:478`, `:819`) and the same eval focus trap for the modal one (`popover.rs:523`).
- **Menus and selects, and the other `cfg` pairs.** The same split: the `web` arm uses the `popover` attribute and the anchored placement helpers in `top_layer/`; the `not(web)` arm renders without them (the select list is a plain always-in-flow `div`, `select/components/list.rs:493`; a menu panel gets no `popover` attribute, `menu_root.rs:313`). Other pairs exist in `tooltip.rs`, `hover_card.rs`, `navbar.rs`, `navigation_menu.rs`, `toast.rs` and `message_scroller.rs`; their differences are not read here.
- **SSR.** The server build must use the same arm as the wasm client or the prerendered markup differs from what hydrates; `check-cfg-axis.sh:3-9` states this as the reason the axis is `web` and not `target_family`.

What a user is told today (checked 2026-11-12): the repository `README.md` and `cli/README.md` never mention `web`; the CLI prints nothing about it on `add` or `init` (`cli/src/add.rs:416-470`); component `docs.md` files do not mention it; the docs-site install pages carry no mention. The one place it appears is the `dioxus-primitives` crate README (`primitives/README.md:38`), in a feature table: "the browser arm of the overlay and listener code", with no statement that an app must enable it. It is documented only there, not defaulted (the crate default is `router`, `primitives/Cargo.toml:45`), and otherwise silent.

This row is the precondition of [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md)'s 3.DJ half: deleting the `not(web)` focus trap is safe only once every browser-engine build is guaranteed to enable `web`.

## Status / trigger

OPEN, found 2026-11-12. Needs the owner's choice between options A, B and C before any code. Found by a read-only check of row 448's "dead code" premise against the source on `bdb41d2a`; the wiring facts above are read from `Cargo.toml` files, `cli/src/manifest.rs` and the smoke script, not from a build, and no desktop webview build has ever been exercised ([row 23](./023-preview-s-desktop-feature-lacks-dioxus.md)).

## Size

M (inferred); A is S, C is S to M, B is M to L.

## Detail

The owner's order of targets is web and the webview now, Blitz later ([row 438](./438-blitz-native-renderer.md), decisions.md 2026-11-11), and "the same components run the same on web or native". The current polarity has the browser arm opt-in and the future target as the default, which is the wrong way round for that order.

- **(A) The CLI enables `web`.** `add` writes `dioxus-primitives` with `--features web` (or adds `web` to the app's own browser-engine features, `web`, `desktop`, `mobile` and `server`, the way the starter does) and the install smoke builds the way a user would, asserting that the browser arm is compiled. Cost: the CLI must know which of the user's features are browser-engine ones, which is a guess for an existing app, and a Blitz app must then opt out by hand. SSG and server: unchanged, since the server feature carries `web` already; an existing app's server feature still needs it added by the CLI or by hand. Starter: unchanged. Row 448: unblocked for CLI-installed apps only; an app that edits its own Cargo.toml can still drop `web`.
- **(B) Invert the feature.** The browser arm is the default; Blitz is the opt-in (a `native` or `blitz` feature that selects the plain arm), so the common case is correct with no wiring and the missing-feature failure class is gone by construction. Cost: every `cfg(feature = "web")` site flips (the `check-cfg-axis` gate and `check-primitives-features` change with it), `time?/wasm-bindgen` stays on a separate feature, the preview's `native` feature gains the opt-in, and the starter drops its `dioxus-primitives/web` lines. SSG and server: correct by default, since server and client both take the default arm, and a Blitz build is the only one that must say so. Row 448: safe as soon as the flip lands, because the focus trap then exists only where `native` is on, which is the case it was written for. Matches the owner's order of targets. Recommended.
- **(C) A check that refuses the ambiguous combination.** A `compile_error!` when neither `web` nor an explicit `blitz` marker is present, or an install-time check in the CLI that refuses to write the dependency without a browser-engine feature. Cost: a hard error for apps that today compile, and a marker feature that every consumer must learn; it states the contract but does not make the common case right. SSG and server: unchanged, as the marker is per build. Starter: gains the marker. Row 448: unblocked once the check lands. A reasonable second step on top of A, weak alone.

**Recommendation:** B, because it makes the web and the webview, the targets the owner wants first, correct with no wiring, removes the whole class (the missing feature) instead of documenting it, and leaves a single explicit opt-in for Blitz when row 438 starts; A is the cheap stopgap if B is too large before row 448. Whichever is chosen, the primitives README feature table and the install docs state the contract in one sentence.

**Proof when closed:** `scripts/registry-install-smoke.sh` builds a CLI-installed app for the web target (not made from the starter) and asserts that the browser arm is compiled (for example, a built marker or a grep of the rendered dialog for `<dialog`), and a gate fails when an app-facing Cargo.toml (the starter, the preview) or the CLI's dependency line wires the feature differently from the contract. Row 448's 3.DJ half then has its precondition.
