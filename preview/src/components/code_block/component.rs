use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Check, Copy};
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

/// The tone of a [`CodeBlock`].
#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum CodeBlockVariant {
    /// A bordered, neutral block.
    #[default]
    Default,
    /// A destructive tone for an error, a failing command or a removed file.
    Destructive,
}

impl CodeBlockVariant {
    pub fn class(&self) -> &'static str {
        match self {
            CodeBlockVariant::Default => "default",
            CodeBlockVariant::Destructive => "destructive",
        }
    }
}

/// The type size of a [`CodeBlock`].
#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum CodeBlockSize {
    /// `text-sm`.
    #[default]
    Default,
    /// `text-xs`, tighter padding: for a block inside a message.
    Sm,
}

impl CodeBlockSize {
    pub fn class(&self) -> &'static str {
        match self {
            CodeBlockSize::Default => "default",
            CodeBlockSize::Sm => "sm",
        }
    }
}

/// How a [`CodeBlockLine`] is marked, for a diff or a call-out.
#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum CodeBlockLineVariant {
    /// An ordinary line.
    #[default]
    Default,
    /// A line added by a diff (a `+` in the gutter).
    Added,
    /// A line removed by a diff (a `−` in the gutter).
    Removed,
    /// A line to draw the reader's eye to.
    Highlighted,
}

impl CodeBlockLineVariant {
    pub fn class(&self) -> &'static str {
        match self {
            CodeBlockLineVariant::Default => "default",
            CodeBlockLineVariant::Added => "added",
            CodeBlockLineVariant::Removed => "removed",
            CodeBlockLineVariant::Highlighted => "highlighted",
        }
    }
}

/// A block of code: an optional header (filename, language, actions), the code in a
/// scrollable `<pre>` and an optional footer. It owns the frame only. It bundles no
/// highlighter: render each highlighted line as a [`CodeBlockLine`] child of
/// [`CodeBlockContent`] with whatever highlighter the app already uses, or pass plain `code`.
///
/// Presentational by default (no role). The scrollable code is a focusable region
/// (`tabindex="0"`) so a keyboard user can scroll it.
#[component]
pub fn CodeBlock(
    /// The tone of the block.
    #[props(default)]
    variant: CodeBlockVariant,
    /// The type size and padding.
    #[props(default)]
    size: CodeBlockSize,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The header, the content and the footer.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-code-block",
        "data-slot": "code-block",
    });
    // `data-variant` and `data-size` reflect this component's own typed props -- owned-wins.
    let owned = attributes!(div {
        "data-variant": variant.class(),
        "data-size": size.class(),
    });
    let merged = merge_attributes(vec![base, attributes, owned]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/code_block/style.css") }
        div { ..merged, {children} }
    }
}

/// The row above the code: a [`CodeBlockTitle`] and, at the end, [`CodeBlockActions`].
#[component]
pub fn CodeBlockHeader(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The title and the actions.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-code-block-header",
        "data-slot": "code-block-header",
    });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/code_block/style.css") }
        div { ..merged, {children} }
    }
}

/// The filename or language label. Truncates; put a decorative icon inside it if you like.
#[component]
pub fn CodeBlockTitle(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The label.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-code-block-title",
        "data-slot": "code-block-title",
    });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/code_block/style.css") }
        div { ..merged, {children} }
    }
}

/// The buttons at the end of the header (a [`CodeBlockCopyButton`]). As a direct child of
/// [`CodeBlock`] (no header) they float over the top corner and show on hover or focus.
#[component]
pub fn CodeBlockActions(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The action buttons.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-code-block-actions",
        "data-slot": "code-block-actions",
    });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/code_block/style.css") }
        div { ..merged, {children} }
    }
}

/// A button that copies `value` to the clipboard. Its accessible name flips from "Copy" to
/// "Copied" for about two seconds and a visually hidden live region announces it, so the
/// result reaches a screen reader and not only the icon. Falls back to `execCommand` where
/// the async clipboard is unavailable (an insecure origin).
#[component]
pub fn CodeBlockCopyButton(
    /// The text to copy: the verbatim code, not the highlighted markup.
    #[props(into)]
    value: String,
    /// The button's look. A ghost icon button by default.
    #[props(default = ButtonVariant::Ghost)]
    variant: ButtonVariant,
    /// The button's size.
    #[props(default = ButtonSize::IconXs)]
    size: ButtonSize,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let mut copied = use_signal(|| false);
    let base = attributes!(button {
        "data-slot": "code-block-copy-button",
    });
    let owned = attributes!(button {
        "data-copied": if copied() { Some("true") } else { None },
        "aria-label": if copied() { "Copied" } else { "Copy" },
    });
    let merged = merge_attributes(vec![base, attributes, owned]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/code_block/style.css") }
        Button {
            variant,
            size,
            attributes: merged,
            onclick: move |_| {
                let text = value.clone();
                spawn(async move {
                    let mut clip = document::eval(
                        r#"
                        const text = await dioxus.recv();
                        try {
                            await navigator.clipboard.writeText(text);
                        } catch (e) {
                            const area = document.createElement('textarea');
                            area.value = text;
                            area.setAttribute('readonly', '');
                            area.style.position = 'fixed';
                            area.style.opacity = '0';
                            document.body.appendChild(area);
                            area.select();
                            document.execCommand('copy');
                            area.remove();
                        }
                        dioxus.send(true);
                        await new Promise((resolve) => setTimeout(resolve, 2000));
                        dioxus.send(true);
                        "#,
                    );
                    let _ = clip.send(text);
                    if clip.recv::<bool>().await.is_ok() {
                        copied.set(true);
                        let _ = clip.recv::<bool>().await;
                        copied.set(false);
                    }
                });
            },
            if copied() {
                Check { size: "1rem", "aria-hidden": "true" }
            } else {
                Copy { size: "1rem", "aria-hidden": "true" }
            }
        }
        span {
            class: "dx-code-block-sr-only",
            "aria-live": "polite",
            if copied() {
                "Copied"
            }
        }
    }
}

/// The code. Pass `code` for plain text (split into one [`CodeBlockLine`] per line), or pass
/// children (your highlighter's lines, each a [`CodeBlockLine`]) for coloured output; `code`
/// wins when both are given. Always `dir="ltr"`: code does not mirror.
#[component]
pub fn CodeBlockContent(
    /// Plain code, rendered as unhighlighted lines.
    #[props(into, default)]
    code: Option<String>,
    /// Wrap long lines instead of scrolling sideways.
    #[props(default)]
    wrap: bool,
    /// Show a line number in a gutter, drawn by CSS (not selectable, not announced).
    #[props(default)]
    line_numbers: bool,
    /// The number of the first line, when the snippet starts mid-file.
    #[props(default = 1)]
    start_line: i64,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// Pre-highlighted lines. Ignored when `code` is given.
    #[props(default)]
    children: Element,
) -> Element {
    let base = attributes!(pre {
        class: "dx-code-block-content",
        "data-slot": "code-block-content",
        tabindex: "0",
    });
    let owned = attributes!(pre {
        dir: "ltr",
        "data-wrap": if wrap { Some("true") } else { None },
        "data-numbered": if line_numbers { Some("true") } else { None },
        style: format!("counter-reset: line {};", start_line - 1),
    });
    let merged = merge_attributes(vec![base, attributes, owned]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/code_block/style.css") }
        pre { ..merged,
            code {
                class: "dx-code-block-code",
                "data-slot": "code-block-code",
                if let Some(code) = code {
                    for (i , line) in code.strip_suffix('\n').unwrap_or(&code).split('\n').enumerate() {
                        CodeBlockLine { key: "{i}",
                            if line.is_empty() {
                                "\n"
                            } else {
                                "{line}"
                            }
                        }
                    }
                } else {
                    {children}
                }
            }
        }
    }
}

/// One line of code: put the line's (highlighted) spans inside. Give an empty line a `"\n"`
/// child so copying a selection keeps blank lines.
#[component]
pub fn CodeBlockLine(
    /// How the line is marked.
    #[props(default)]
    variant: CodeBlockLineVariant,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The line's content.
    children: Element,
) -> Element {
    let base = attributes!(span {
        class: "dx-code-block-line",
        "data-slot": "code-block-line",
    });
    let owned = attributes!(span { "data-variant": variant.class() });
    let merged = merge_attributes(vec![base, attributes, owned]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/code_block/style.css") }
        span { ..merged,
            span { class: "dx-code-block-line-content", {children} }
        }
    }
}

/// The row below the code (a caption, the exit status, a "Run" action).
#[component]
pub fn CodeBlockFooter(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The footer content.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-code-block-footer",
        "data-slot": "code-block-footer",
    });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/code_block/style.css") }
        div { ..merged, {children} }
    }
}
