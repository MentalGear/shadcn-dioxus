use super::*;

#[derive(Clone, PartialEq)]
pub struct HighlightedCode {
    pub source: HighlightedSource,
}

/// A code listing in the docs prose. `.dx-docs-prose pre` scrolls sideways, and a scrollable region must
/// be reachable from the keyboard (axe `scrollable-region-focusable`), so every listing in the docs
/// goes through this one component instead of a bare `pre` (same reason `CodeBlock` is focusable).
#[component]
pub(crate) fn DocsPre(children: Element) -> Element {
    rsx! {
        pre { tabindex: "0", {children} }
    }
}

#[component]
pub(crate) fn CodeBlock(source: HighlightedCode) -> Element {
    rsx! {
        div {
            class: "dx-code-block",
            tabindex: "0",
            PreviewCode { source: source.source }
        }
        CopyButton { position: "absolute", top: "0.5em", right: "0.5em" }
    }
}

/// The line numbers shown beside a source listing: "1\n2\n...\nN", N being the number of lines
/// `Code` renders (it drops trailing newlines, so a file's final newline is not a line).
pub(crate) fn line_numbers(source: &str) -> String {
    let lines = source.trim_end_matches('\n').split('\n').count();
    (1..=lines)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// A highlighted listing with its line numbers in a gutter. The gutter is a separate, `aria-hidden`,
/// `user-select: none` column (see `.dx-code-gutter`), so selecting and copying code never picks the
/// numbers up, and the Copy buttons hand over the source text untouched. It uses the code's own font
/// metrics and does not wrap, so each number stays level with its line.
#[component]
pub(crate) fn PreviewCode(source: HighlightedSource) -> Element {
    let numbers = line_numbers(source.source());
    rsx! {
        div {
            class: "dx-preview-code-theme",
            tabindex: "0",
            pre { class: "dx-code-gutter", "aria-hidden": "true", "data-slot": "code-gutter", "{numbers}" }
            Code {
                src: source,
                theme: CodeTheme::system(Theme::GITHUB_LIGHT, Theme::GITHUB_DARK),
            }
        }
    }
}

/// The Style tab's CSS source for one component file (`style.css`, or a
/// Block-kind variant's `demo.css`).
///
/// In release/SSG builds this carries the text highlighted (and embedded)
/// at compile time via `dioxus_code::code!()`, exactly like
/// `HighlightedCode`. In debug
/// builds (`dx serve`'s own dev loop) `code!()` is deliberately NOT used:
/// it expands to `include_str!(path)`, which makes rustc track the `.css`
/// file as a source dependency of this crate. `dx serve`'s file watcher
/// checks that dependency list (dioxus-cli's
/// `serve/runner.rs::handle_file_change`, via the compiled artifact's own
/// rustc dep-info `.d` file) and, finding the edited file listed there,
/// classifies the edit as `needs_full_rebuild` -- a full ~30-60s rebuild,
/// instead of the sub-second asset hot-reload every *other* `style.css`
/// edit already gets from its own separate `asset!()` stylesheet link
/// (`hotreload_bundled_assets`, checked first, but overridden once
/// `needs_full_rebuild` is also true for that same file). The measurement
/// and the full mechanism: `dev-docs/dev-loop.md`'s CSS
/// section. Debug builds instead carry only the file's own bundled
/// `asset!()` URL -- `asset!()` does not embed the file's text at compile
/// time, so referencing it costs nothing extra; `CssCodeBlock` below
/// fetches and highlights that URL's text lazily, client-side, the first
/// time the Style tab is rendered.
#[derive(Clone, PartialEq)]
pub(crate) struct CssHighlight {
    /// Pre-highlighted content -- release/SSG builds only.
    #[cfg(not(debug_assertions))]
    pub(crate) embedded: HighlightedCode,
    /// This file's own bundled asset URL -- debug builds only.
    #[cfg(debug_assertions)]
    pub(crate) asset: Asset,
}

/// Renders a CSS Style tab from a [`CssHighlight`]. Release/SSG builds
/// show the compile-time-highlighted text directly, identical to
/// `CodeBlock`.
#[cfg(not(debug_assertions))]
#[component]
pub(crate) fn CssCodeBlock(source: CssHighlight) -> Element {
    rsx! {
        CodeBlock { source: source.embedded }
    }
}

/// Debug builds' half of [`CssCodeBlock`]: fetch the CSS from its own
/// asset URL and highlight it at runtime instead of at compile time --
/// see `CssHighlight`'s doc comment for why.
#[cfg(debug_assertions)]
#[component]
pub(crate) fn CssCodeBlock(source: CssHighlight) -> Element {
    rsx! {
        LazyCssCodeBlock { asset: source.asset }
    }
}

/// Fetches `asset`'s own text over HTTP -- the same URL its
/// `document::Link`/`document::Stylesheet` sibling already loads as a live
/// stylesheet -- and highlights it client-side once it arrives, so a
/// `style.css` edit is visible here exactly like the live stylesheet
/// already is: via `dx serve`'s asset hot-reload, not a crate rebuild.
#[cfg(debug_assertions)]
#[component]
pub(crate) fn LazyCssCodeBlock(asset: Asset) -> Element {
    let url = asset.to_string();
    let css = use_resource(move || {
        let url = url.clone();
        async move { fetch_asset_text(url).await }
    });

    // Fully qualified rather than imported: a bare `Language` would be ambiguous with other
    // language types in scope.
    let placeholder = |text: &'static str| HighlightedCode {
        source: HighlightedSource::from_static_parts(text, dioxus_code::Language::Css, &[]),
    };

    // Cloned out of the resource's read guard into an owned value up front
    // (rather than matching on `&*css.read()` directly) so the guard drops
    // immediately, before any of the `rsx!` arms below run -- avoids tying
    // the returned `Element` to that guard's borrow.
    let state: Option<Option<String>> = css.read().clone();

    match state {
        Some(Some(text)) => rsx! {
            CodeBlock {
                source: HighlightedCode {
                    source: dioxus_code::SourceCode::new(dioxus_code::Language::Css, text).into(),
                },
            }
        },
        Some(None) => rsx! { CodeBlock { source: placeholder("/* failed to load style.css */") } },
        None => rsx! { CodeBlock { source: placeholder("/* loading style.css... */") } },
    }
}

/// One-shot fetch of a same-origin asset's raw text, used only by
/// [`LazyCssCodeBlock`] (debug builds). Mirrors this codebase's other
/// one-shot `document::eval` helpers (e.g. `primitives/src/input_otp.rs`'s
/// `snap_caret_to_slot`) -- a scoped async JS snippet, not a persistent
/// listener.
#[cfg(debug_assertions)]
pub(crate) async fn fetch_asset_text(url: String) -> Option<String> {
    let mut eval = document::eval(
        r#"
        const url = await dioxus.recv();
        try {
            const res = await fetch(url);
            dioxus.send(res.ok ? await res.text() : null);
        } catch (e) {
            dioxus.send(null);
        }
        "#,
    );
    let _ = eval.send(url);
    eval.recv::<Option<String>>().await.ok().flatten()
}

/// The docs' copy-to-clipboard button for a code block: the library's own ghost icon `Button`.
///
/// The copy runs from an inline `onclick` attribute (not a Rust handler) so it works before the
/// wasm hydrates. The same script flips `data-copied`, and
/// `main.css` swaps the two icons on it (a Rust-side signal would need its own `onclick`, which
/// `merge_attributes` would let the inline string replace). Placement over the code block comes
/// from the caller's `attributes` (`position`/`top`/`right`).
#[component]
pub(crate) fn CopyButton(#[props(extends=GlobalAttributes)] attributes: Vec<Attribute>) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconSm,
            attributes,
            class: "dx-copy-button",
            r#type: "button",
            aria_label: "Copy code",
            "onclick": "const b = this; const visiblePre = Array.from(b.parentNode.querySelectorAll('pre:not(.dx-code-gutter)')).find((pre) => pre.offsetParent !== null); navigator.clipboard.writeText(visiblePre ? visiblePre.innerText : Array.from(b.parentNode.childNodes).filter((node) => node !== b).map((node) => node.textContent).join('').trim()); b.dataset.copied = 'true';",
            Copy { class: "dx-copy-icon" }
            Check { class: "dx-check-icon" }
        }
    }
}

#[component]
pub(crate) fn ComponentCode(
    rs_highlighted: HighlightedCode,
    css_highlighted: CssHighlight,
    #[props(default = ComponentType::Normal)] component_type: ComponentType,
) -> Element {
    rsx! {
        Tabs {
            default_value: "main.rs",
            border_bottom_left_radius: "0.5rem",
            border_bottom_right_radius: "0.5rem",
            horizontal: true,
            width: "100%",
            TabList {
                TabTrigger { value: "main.rs", "main.rs" }
                TabTrigger { value: "style.css", "style.css" }
                if component_type != ComponentType::Block {
                    TabTrigger { value: "dx-components-theme.css", "dx-components-theme.css" }
                }
            }
            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                justify_content: "center",
                align_items: "center",
                TabContent {
                    padding: 0,
                    value: "main.rs",
                    width: "100%",
                    position: "relative",
                    CodeBlock { source: rs_highlighted }
                }
                TabContent {
                    padding: 0,
                    value: "style.css",
                    width: "100%",
                    position: "relative",
                    CssCodeBlock { source: css_highlighted }
                }
                if component_type != ComponentType::Block {
                    TabContent {
                        padding: 0,
                        value: "dx-components-theme.css",
                        width: "100%",
                        position: "relative",
                        CssCodeBlock { source: THEME_CSS }
                    }
                }
            }
        }
    }
}
