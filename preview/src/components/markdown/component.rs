//! `Markdown`: renders model output (CommonMark + tables, strikethrough, task lists) as Dioxus nodes.
//!
//! Safe by construction, not by filtering: the parser's event stream is folded into [`Node`], a small
//! closed enum with NO raw-HTML variant, and `Node` is the only thing the renderer reads. So
//!
//! * raw HTML (`Event::Html` / `InlineHtml`, `<script>`, `<img onerror=..>`) can only become a text
//!   node, which the renderer escapes; there is no `dangerous_inner_html` anywhere in this file;
//! * a link target exists in the tree only as a [`SafeHref`], a type whose sole constructor is
//!   [`safe_href`] (http, https, mailto, tel and relative only), so `javascript:`, `data:` and
//!   `vbscript:` URLs cannot reach an `href` or `src`;
//! * images are not rendered unless the consumer opts in with `allow_images`, because a model can
//!   exfiltrate data through the query string of an image the browser loads by itself;
//! * heading ids, classes and attributes, metadata blocks, footnotes and math are not parser options
//!   here, so there is no way to inject an `id` or `class` from the text.
//!
//! The streaming heal pass ([`heal_markdown`]) and the bare-URL autolink pass are plain `&str` / tree
//! functions of this file, unit-tested below without a browser.

use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use pulldown_cmark::{Alignment, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

// ---------------------------------------------------------------------------------------------
// Links: the one allowlist
// ---------------------------------------------------------------------------------------------

/// What kind of destination a [`SafeHref`] is.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum HrefKind {
    /// `http:`, `https:` or protocol-relative `//host`: leaves the app, so it opens in a new tab.
    External,
    /// `mailto:` or `tel:`: handed to the OS, no new tab.
    Contact,
    /// A path, `?query` or `#fragment` on the current origin.
    Relative,
}

/// A link or image target that passed the allowlist. The field is private and [`safe_href`] is the
/// only constructor, so holding a `SafeHref` is the proof the scheme is `http`, `https`, `mailto`,
/// `tel` or relative.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SafeHref {
    url: String,
    kind: HrefKind,
}

impl SafeHref {
    /// The URL text, exactly as written (after trimming surrounding whitespace).
    pub fn as_str(&self) -> &str {
        &self.url
    }

    /// Whether the link leaves the app (opens in a new tab with `rel="noopener noreferrer"`).
    pub fn is_external(&self) -> bool {
        self.kind == HrefKind::External
    }

    /// Whether the target may be loaded as an image: web and same-origin URLs, never `mailto:`/`tel:`.
    fn allows_image(&self) -> bool {
        self.kind != HrefKind::Contact
    }
}

/// The link policy: `Some` only for `http`, `https`, `mailto`, `tel` and relative targets.
///
/// Anything else (`javascript:`, `data:`, `vbscript:`, `file:`, an unknown scheme) is `None`, and so
/// is any target with whitespace, a control character or a backslash anywhere in it: browsers strip
/// tabs and newlines inside a scheme (`java\tscript:`) and read `\` as `/` (`/\evil.example`), so the
/// check rejects the whole class instead of trying to normalise it.
pub fn safe_href(raw: &str) -> Option<SafeHref> {
    let s = raw.trim();
    if s.is_empty()
        || s.chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
    {
        return None;
    }
    let url = s.to_string();
    match s.find([':', '/', '?', '#']) {
        Some(p) if s.as_bytes()[p] == b':' => {
            let rest = &s[p + 1..];
            match s[..p].to_ascii_lowercase().as_str() {
                "http" | "https" => {
                    let host = rest.strip_prefix("//")?;
                    (!host.is_empty() && !host.starts_with('/')).then_some(SafeHref {
                        url,
                        kind: HrefKind::External,
                    })
                }
                "mailto" | "tel" => (!rest.is_empty()).then_some(SafeHref {
                    url,
                    kind: HrefKind::Contact,
                }),
                _ => None,
            }
        }
        _ if s.starts_with("//") => (s.len() > 2 && !s[2..].starts_with('/')).then_some(SafeHref {
            url,
            kind: HrefKind::External,
        }),
        _ => Some(SafeHref {
            url,
            kind: HrefKind::Relative,
        }),
    }
}

// ---------------------------------------------------------------------------------------------
// The allowlisted tree
// ---------------------------------------------------------------------------------------------

/// Every element this component can emit. A parser event with no row here (raw HTML, math,
/// footnotes) never becomes an element.
#[derive(Clone, PartialEq, Debug)]
enum Kind {
    Paragraph,
    Heading(u8),
    Quote,
    List(Option<u64>),
    Item,
    /// A fenced or indented code block; the children are one `Text`.
    CodeBlock(Option<String>),
    Table,
    TableHead,
    TableRow,
    TableCell(Alignment, bool),
    Emphasis,
    Strong,
    Strike,
    Link(Option<SafeHref>, Option<String>),
    Image(Option<SafeHref>, Option<String>),
}

#[derive(Clone, PartialEq, Debug)]
enum Node {
    Text(String),
    Code(String),
    SoftBreak,
    HardBreak,
    Rule,
    Task(bool),
    Element(Kind, Vec<Node>),
}

/// Plain text of a subtree (an image's alt text, a link label).
fn plain_text(nodes: &[Node], out: &mut String) {
    for node in nodes {
        match node {
            Node::Text(t) | Node::Code(t) => out.push_str(t),
            Node::SoftBreak | Node::HardBreak => out.push(' '),
            Node::Element(_, children) => plain_text(children, out),
            Node::Rule | Node::Task(_) => {}
        }
    }
}

fn push_text(nodes: &mut Vec<Node>, text: &str) {
    if let Some(Node::Text(last)) = nodes.last_mut() {
        last.push_str(text);
    } else {
        nodes.push(Node::Text(text.to_string()));
    }
}

/// The info string's first word as a language name, or `None` unless it is a short identifier
/// (`rust`, `c++`, `c#`, `js-x`): it becomes a class name, so it is restricted to a safe alphabet.
fn code_language(info: &str) -> Option<String> {
    let word = info.split_whitespace().next()?;
    let word = word.strip_prefix('{').unwrap_or(word);
    let ok = !word.is_empty()
        && word.len() <= 32
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '#' | '.' | '-'));
    ok.then(|| word.to_ascii_lowercase())
}

fn parse_options() -> Options {
    // Deliberately short: no footnotes, heading attributes, metadata blocks, math, definition lists,
    // smart punctuation or GFM alerts. Each of those is surface a model could use to inject.
    Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS
}

/// Folds the parser's events into the allowlisted tree.
fn build_tree(source: &str) -> Vec<Node> {
    let mut stack: Vec<(Kind, Vec<Node>)> = Vec::new();
    let mut root: Vec<Node> = Vec::new();
    // Alignment of the table being built, and the column of the next cell.
    let mut aligns: Vec<Alignment> = Vec::new();
    let mut column = 0usize;
    let mut in_head = false;

    macro_rules! top {
        () => {
            match stack.last_mut() {
                Some((_, children)) => children,
                None => &mut root,
            }
        };
    }

    for event in Parser::new_ext(source, parse_options()) {
        match event {
            Event::Start(tag) => {
                let kind = match tag {
                    Tag::Paragraph => Kind::Paragraph,
                    Tag::Heading { level, .. } => Kind::Heading(level as u8),
                    Tag::BlockQuote(_) => Kind::Quote,
                    Tag::CodeBlock(CodeBlockKind::Fenced(info)) => {
                        Kind::CodeBlock(code_language(&info))
                    }
                    Tag::CodeBlock(CodeBlockKind::Indented) => Kind::CodeBlock(None),
                    // A raw HTML block: its `Event::Html` children are text, so it is a paragraph
                    // of literal source.
                    Tag::HtmlBlock => Kind::Paragraph,
                    Tag::List(start) => Kind::List(start),
                    Tag::Item => Kind::Item,
                    Tag::Table(a) => {
                        aligns = a;
                        Kind::Table
                    }
                    Tag::TableHead => {
                        column = 0;
                        in_head = true;
                        Kind::TableHead
                    }
                    Tag::TableRow => {
                        column = 0;
                        Kind::TableRow
                    }
                    Tag::TableCell => {
                        let align = aligns.get(column).copied().unwrap_or(Alignment::None);
                        column += 1;
                        Kind::TableCell(align, in_head)
                    }
                    Tag::Emphasis => Kind::Emphasis,
                    Tag::Strong => Kind::Strong,
                    Tag::Strikethrough => Kind::Strike,
                    Tag::Link {
                        dest_url, title, ..
                    } => Kind::Link(
                        safe_href(&dest_url),
                        (!title.is_empty()).then(|| title.to_string()),
                    ),
                    Tag::Image {
                        dest_url, title, ..
                    } => Kind::Image(
                        safe_href(&dest_url),
                        (!title.is_empty()).then(|| title.to_string()),
                    ),
                    // Not enabled as options, so the parser never opens them; if a future version
                    // does, their content still renders, as a paragraph.
                    _ => Kind::Paragraph,
                };
                stack.push((kind, Vec::new()));
            }
            Event::End(end) => {
                if end == TagEnd::TableHead {
                    in_head = false;
                }
                if let Some((kind, children)) = stack.pop() {
                    top!().push(Node::Element(kind, children));
                }
            }
            // Raw HTML, math and footnote references are text. The renderer escapes text.
            Event::Text(t)
            | Event::Html(t)
            | Event::InlineHtml(t)
            | Event::InlineMath(t)
            | Event::DisplayMath(t) => push_text(top!(), &t),
            Event::FootnoteReference(label) => push_text(top!(), &format!("[^{label}]")),
            Event::Code(c) => top!().push(Node::Code(c.to_string())),
            Event::SoftBreak => top!().push(Node::SoftBreak),
            Event::HardBreak => top!().push(Node::HardBreak),
            Event::Rule => top!().push(Node::Rule),
            Event::TaskListMarker(checked) => top!().push(Node::Task(checked)),
        }
    }
    // An unbalanced stream cannot happen, but never drop content if it does.
    while let Some((kind, children)) = stack.pop() {
        top!().push(Node::Element(kind, children));
    }
    autolink(&mut root);
    root
}

// ---------------------------------------------------------------------------------------------
// Bare-URL autolinks (pulldown-cmark has no GFM autolink literal): our own pass over text nodes
// ---------------------------------------------------------------------------------------------

/// Turns bare `http://`, `https://` and `www.` URLs in text into links, outside code, links and
/// images. The URL still goes through [`safe_href`].
fn autolink(nodes: &mut Vec<Node>) {
    let mut out = Vec::with_capacity(nodes.len());
    for node in std::mem::take(nodes) {
        match node {
            Node::Text(text) => split_urls(&text, &mut out),
            Node::Element(kind, mut children) => {
                if !matches!(kind, Kind::Link(..) | Kind::Image(..) | Kind::CodeBlock(_)) {
                    autolink(&mut children);
                }
                out.push(Node::Element(kind, children));
            }
            other => out.push(other),
        }
    }
    *nodes = out;
}

fn split_urls(text: &str, out: &mut Vec<Node>) {
    let mut plain_from = 0;
    let mut i = 0;
    while i < text.len() {
        let rest = &text[i..];
        let at_boundary = text[..i]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric());
        let is_start = at_boundary
            && (rest.starts_with("http://")
                || rest.starts_with("https://")
                || rest.starts_with("www."));
        if !is_start {
            i += rest.chars().next().map_or(1, char::len_utf8);
            continue;
        }
        let mut end = i + rest
            .find(|c: char| c.is_whitespace() || c == '<')
            .unwrap_or(rest.len());
        // Trailing punctuation belongs to the sentence, and so does an unmatched closing bracket.
        loop {
            let candidate = &text[i..end];
            let Some(last) = candidate.chars().next_back() else {
                break;
            };
            let unmatched = |open: char, close: char| {
                candidate.matches(close).count() > candidate.matches(open).count()
            };
            let trim = matches!(
                last,
                '.' | ',' | ':' | ';' | '!' | '?' | '\'' | '"' | '*' | '_' | '~'
            ) || (last == ')' && unmatched('(', ')'))
                || (last == ']' && unmatched('[', ']'));
            if !trim {
                break;
            }
            end -= last.len_utf8();
        }
        let url = &text[i..end];
        let after_scheme = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .or_else(|| url.strip_prefix("www."))
            .unwrap_or("");
        let linked = after_scheme.chars().any(char::is_alphanumeric);
        let href = if url.starts_with("www.") {
            format!("http://{url}")
        } else {
            url.to_string()
        };
        match (linked, safe_href(&href)) {
            (true, Some(href)) => {
                if plain_from < i {
                    out.push(Node::Text(text[plain_from..i].to_string()));
                }
                out.push(Node::Element(
                    Kind::Link(Some(href), None),
                    vec![Node::Text(url.to_string())],
                ));
                plain_from = end;
                i = end;
            }
            _ => i += rest.chars().next().map_or(1, char::len_utf8),
        }
    }
    if plain_from < text.len() {
        out.push(Node::Text(text[plain_from..].to_string()));
    }
}

// ---------------------------------------------------------------------------------------------
// The streaming heal pass
// ---------------------------------------------------------------------------------------------

/// Makes the text of a half-finished reply safe to parse, so a stream never flashes `**` or an
/// unrendered `[label](https://exa`: an unclosed code span or emphasis is closed, a link whose
/// target has not arrived yet shows only its label, and a closing fence typed one backtick at a time
/// does not appear as content. Only the last paragraph can be unfinished, so only it is touched.
/// Total on any input and idempotent: `heal_markdown(heal_markdown(s)) == heal_markdown(s)`.
pub fn heal_markdown(source: &str) -> String {
    // Which fence is open at the end, and where the last paragraph starts (a byte offset on a line
    // boundary).
    let mut fence: Option<(char, usize)> = None;
    let mut block_start = 0usize;
    let mut offset = 0usize;
    for line in source.split_inclusive('\n') {
        let line_end = offset + line.len();
        let body = line.trim_end_matches(['\n', '\r']);
        let indent = body.len() - body.trim_start_matches(' ').len();
        let trimmed = body.trim_start_matches(' ');
        let run_char = trimmed.chars().next().filter(|c| matches!(c, '`' | '~'));
        let run = run_char.map_or(0, |c| trimmed.chars().take_while(|x| *x == c).count());
        match fence {
            Some((c, len)) => {
                if indent <= 3
                    && run_char == Some(c)
                    && run >= len
                    && trimmed[run..].trim().is_empty()
                {
                    fence = None;
                    block_start = line_end;
                }
            }
            None => {
                if indent <= 3
                    && run >= 3
                    && !(run_char == Some('`') && trimmed[run..].contains('`'))
                {
                    fence = Some((run_char.unwrap_or('`'), run));
                } else if body.trim().is_empty() {
                    block_start = line_end;
                }
            }
        }
        offset = line_end;
    }

    if let Some((c, len)) = fence {
        // A closing fence typed so far (`` or `~~) is not content: drop it.
        let last_line_start = source.rfind('\n').map_or(0, |p| p + 1);
        let last = &source[last_line_start..];
        let t = last.trim_start_matches(' ');
        let short = !t.is_empty() && t.len() < len && t.chars().all(|x| x == c);
        if last_line_start > 0 && last.len() - t.len() <= 3 && short {
            return source[..last_line_start].to_string();
        }
        return source.to_string();
    }

    let (head, tail) = source.split_at(block_start.min(source.len()));
    format!("{head}{}", heal_inline(tail))
}

fn is_ws(c: Option<char>) -> bool {
    c.is_none_or(char::is_whitespace)
}

/// Heals one paragraph (see [`heal_markdown`]).
fn heal_inline(tail: &str) -> String {
    let chars: Vec<char> = tail.chars().collect();
    let n = chars.len();
    // Indices (into `chars`) of characters to drop, and the closers to append, innermost last.
    let mut drop = vec![false; n];
    let mut open: Vec<&'static str> = Vec::new();
    let mut brackets: Vec<usize> = Vec::new();
    let mut code_closer: Option<String> = None;

    let mut i = 0;
    while i < n {
        let c = chars[i];
        match c {
            '\\' => i += 2,
            '`' => {
                let mut j = i;
                while j < n && chars[j] == '`' {
                    j += 1;
                }
                let len = j - i;
                // The closing run must have exactly the same length.
                let mut k = j;
                let mut close = None;
                while k < n {
                    if chars[k] == '`' {
                        let mut m = k;
                        while m < n && chars[m] == '`' {
                            m += 1;
                        }
                        if m - k == len {
                            close = Some(m);
                            break;
                        }
                        k = m;
                    } else {
                        k += 1;
                    }
                }
                match close {
                    Some(m) => i = m,
                    None => {
                        if chars[j..].iter().all(|x| x.is_whitespace()) {
                            // Nothing typed inside yet: the run itself is the unfinished part.
                            (i..j).for_each(|x| drop[x] = true);
                        } else {
                            code_closer = Some("`".repeat(len));
                        }
                        break;
                    }
                }
            }
            '[' => {
                brackets.push(i);
                i += 1;
            }
            ']' => {
                let opener = brackets.pop();
                if chars.get(i + 1) == Some(&'(') {
                    match chars[i + 2..].iter().position(|x| *x == ')') {
                        Some(p) => i += p + 3,
                        None => {
                            // `[label](https://exa`: the target is still arriving, show the label.
                            if let Some(o) = opener {
                                drop[o] = true;
                                if o > 0 && chars[o - 1] == '!' {
                                    drop[o - 1] = true;
                                }
                            }
                            (i..n).for_each(|x| drop[x] = true);
                            break;
                        }
                    }
                } else {
                    i += 1;
                }
            }
            '*' | '_' | '~' => {
                let mut j = i;
                while j < n && chars[j] == c {
                    j += 1;
                }
                let len = j - i;
                let prev = i.checked_sub(1).map(|p| chars[p]);
                let next = chars.get(j).copied();
                let mut can_open = !is_ws(next);
                let mut can_close = !is_ws(prev);
                if c == '_' {
                    can_open &= !prev.is_some_and(char::is_alphanumeric);
                    can_close &= !next.is_some_and(char::is_alphanumeric);
                }
                if j == n && !can_close {
                    // An opener typed at the very end with nothing after it yet.
                    (i..j).for_each(|x| drop[x] = true);
                }
                let mut markers: Vec<&'static str> = Vec::new();
                if len >= 2 {
                    markers.push(match c {
                        '*' => "**",
                        '_' => "__",
                        _ => "~~",
                    });
                }
                if len % 2 == 1 && c != '~' {
                    markers.push(if c == '*' { "*" } else { "_" });
                }
                for marker in markers {
                    if let Some(pos) = open.iter().rposition(|m| *m == marker) {
                        if can_close {
                            open.remove(pos);
                        }
                    } else if can_open {
                        open.push(marker);
                    }
                }
                i = j;
            }
            _ => i += 1,
        }
    }
    // An opener with no `]` yet (`[label`): the bracket is the unfinished part.
    for o in brackets {
        if !drop[o] {
            drop[o] = true;
            if o > 0 && chars[o - 1] == '!' {
                drop[o - 1] = true;
            }
        }
    }

    let mut out: String = chars
        .iter()
        .zip(&drop)
        .filter(|(_, d)| !**d)
        .map(|(c, _)| *c)
        .collect();
    if code_closer.is_none() && open.is_empty() {
        return out;
    }
    // A closing delimiter cannot follow whitespace.
    if code_closer.is_none() {
        out.truncate(out.trim_end().len());
    }
    if let Some(closer) = code_closer {
        out.push_str(&closer);
    }
    for marker in open.iter().rev() {
        out.push_str(marker);
    }
    out
}

// ---------------------------------------------------------------------------------------------
// The component
// ---------------------------------------------------------------------------------------------

/// What a highlighter receives for one fenced or indented code block.
#[derive(Clone, PartialEq, Debug)]
pub struct CodeBlock {
    /// The info string's first word when it is a plain identifier (`rust`, `c++`), lowercased.
    pub language: Option<String>,
    /// The code, verbatim.
    pub code: String,
}

#[derive(Clone, Copy)]
struct Ctx {
    allow_images: bool,
    highlight: Option<Callback<CodeBlock, Element>>,
}

/// Model output as Markdown. Pass `streaming: true` while the text is still arriving; the unfinished
/// tail is healed before parsing (see [`heal_markdown`]).
///
/// Everything is rendered from an allowlisted tree: raw HTML shows as literal text, links only use
/// `http`, `https`, `mailto`, `tel` or relative targets (external links open in a new tab with
/// `rel="noopener noreferrer"`), and images are NOT loaded unless `allow_images` is set. Code blocks
/// are a plain `<pre><code class="language-x">`; pass `highlight` to render a block yourself.
#[component]
pub fn Markdown(
    /// The Markdown text.
    #[props(into)]
    source: String,
    /// The text is still being generated: heal an unfinished tail so it renders sanely.
    #[props(default)]
    streaming: bool,
    /// Load images from the text. Off by default: a model can encode data in an image URL.
    #[props(default)]
    allow_images: bool,
    /// Renders one code block. Receives the language and the code; the result is placed inside the
    /// block's `<pre>`. Without it the block is a plain `<code>` with a `language-x` class.
    highlight: Option<Callback<CodeBlock, Element>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(div {
        class: "dx-markdown",
        "data-slot": "markdown",
    });
    let owned = attributes!(div {
        "data-streaming": if streaming { "true" } else { "false" },
    });
    let merged = merge_attributes(vec![base, attributes, owned]);
    let healed;
    let text = if streaming {
        healed = heal_markdown(&source);
        healed.as_str()
    } else {
        source.as_str()
    };
    let tree = build_tree(text);
    let ctx = Ctx {
        allow_images,
        highlight,
    };
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/markdown/style.css") }
        div { ..merged, {render_nodes(&tree, ctx)} }
    }
}

fn render_nodes(nodes: &[Node], ctx: Ctx) -> Element {
    rsx! {
        for node in nodes {
            {render_node(node, ctx)}
        }
    }
}

fn render_node(node: &Node, ctx: Ctx) -> Element {
    match node {
        Node::Text(t) => rsx! { "{t}" },
        Node::Code(c) => rsx! {
            code { class: "dx-markdown-code", "{c}" }
        },
        Node::SoftBreak => rsx! { "\n" },
        Node::HardBreak => rsx! {
            br {}
        },
        Node::Rule => rsx! {
            hr { class: "dx-markdown-rule" }
        },
        Node::Task(checked) => rsx! {
            input {
                class: "dx-markdown-task",
                r#type: "checkbox",
                checked: *checked,
                disabled: true,
                "aria-label": if *checked { "Completed" } else { "Not completed" },
            }
        },
        Node::Element(kind, children) => render_element(kind, children, ctx),
    }
}

fn render_element(kind: &Kind, children: &[Node], ctx: Ctx) -> Element {
    let inner = render_nodes(children, ctx);
    match kind {
        Kind::Paragraph => rsx! {
            p { class: "dx-markdown-p", {inner} }
        },
        Kind::Heading(level) => {
            let class = "dx-markdown-heading";
            match level {
                1 => rsx! {
                    h1 { class, {inner} }
                },
                2 => rsx! {
                    h2 { class, {inner} }
                },
                3 => rsx! {
                    h3 { class, {inner} }
                },
                4 => rsx! {
                    h4 { class, {inner} }
                },
                5 => rsx! {
                    h5 { class, {inner} }
                },
                _ => rsx! {
                    h6 { class, {inner} }
                },
            }
        }
        Kind::Quote => rsx! {
            blockquote { class: "dx-markdown-quote", {inner} }
        },
        Kind::List(Some(start)) => rsx! {
            ol { class: "dx-markdown-list", start: "{start}", {inner} }
        },
        Kind::List(None) => rsx! {
            ul { class: "dx-markdown-list", {inner} }
        },
        Kind::Item => {
            let task = matches!(children.first(), Some(Node::Task(_)));
            rsx! {
                li {
                    class: "dx-markdown-item",
                    "data-task": if task { "true" } else { "false" },
                    {inner}
                }
            }
        }
        Kind::CodeBlock(language) => {
            let mut code = String::new();
            plain_text(children, &mut code);
            let code = code.strip_suffix('\n').unwrap_or(&code).to_string();
            match ctx.highlight {
                Some(highlight) => {
                    let info = CodeBlock {
                        language: language.clone(),
                        code,
                    };
                    rsx! {
                        pre {
                            class: "dx-markdown-pre",
                            "data-language": language.clone(),
                            tabindex: "0",
                            {highlight.call(info)}
                        }
                    }
                }
                None => {
                    let class = match language {
                        Some(l) => format!("dx-markdown-block language-{l}"),
                        None => "dx-markdown-block".to_string(),
                    };
                    rsx! {
                        pre {
                            class: "dx-markdown-pre",
                            "data-language": language.clone(),
                            tabindex: "0",
                            code { class, "{code}" }
                        }
                    }
                }
            }
        }
        Kind::Table => {
            let (head, rows) = match children.split_first() {
                Some((Node::Element(Kind::TableHead, head), rows)) => (head.as_slice(), rows),
                _ => (&[][..], children),
            };
            rsx! {
                div {
                    class: "dx-markdown-table-scroll",
                    role: "region",
                    "aria-label": "Table",
                    tabindex: "0",
                    table { class: "dx-markdown-table",
                        thead {
                            tr { {render_nodes(head, ctx)} }
                        }
                        tbody { {render_nodes(rows, ctx)} }
                    }
                }
            }
        }
        Kind::TableHead => rsx! {
            tr { {inner} }
        },
        Kind::TableRow => rsx! {
            tr { {inner} }
        },
        Kind::TableCell(align, header) => {
            let align = match align {
                // Logical, so a table reads correctly in a right-to-left page too.
                Alignment::None | Alignment::Left => "start",
                Alignment::Center => "center",
                Alignment::Right => "end",
            };
            if *header {
                rsx! {
                    th { class: "dx-markdown-cell", scope: "col", "data-align": align, {inner} }
                }
            } else {
                rsx! {
                    td { class: "dx-markdown-cell", "data-align": align, {inner} }
                }
            }
        }
        Kind::Emphasis => rsx! {
            em { {inner} }
        },
        Kind::Strong => rsx! {
            strong { {inner} }
        },
        Kind::Strike => rsx! {
            del { {inner} }
        },
        Kind::Link(Some(href), title) => {
            let url = href.as_str().to_string();
            if href.is_external() {
                rsx! {
                    a {
                        class: "dx-markdown-link",
                        href: url,
                        title: title.clone(),
                        target: "_blank",
                        rel: "noopener noreferrer",
                        {inner}
                    }
                }
            } else {
                rsx! {
                    a {
                        class: "dx-markdown-link",
                        href: url,
                        title: title.clone(),
                        {inner}
                    }
                }
            }
        }
        // A link the policy refused keeps its label and loses its target.
        Kind::Link(None, _) => rsx! {
            span { class: "dx-markdown-link-inert", "data-blocked": "link", {inner} }
        },
        Kind::Image(src, title) => {
            let mut alt = String::new();
            plain_text(children, &mut alt);
            match src {
                Some(src) if ctx.allow_images && src.allows_image() => rsx! {
                    img {
                        class: "dx-markdown-image",
                        src: src.as_str().to_string(),
                        alt,
                        title: title.clone(),
                        loading: "lazy",
                        decoding: "async",
                        referrerpolicy: "no-referrer",
                    }
                },
                // Images off (the default) or a refused source: the alt text stays, nothing loads.
                _ => rsx! {
                    span { class: "dx-markdown-image-alt", "data-blocked": "image", "{alt}" }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(source: &str, streaming: bool, allow_images: bool) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct P {
            source: String,
            streaming: bool,
            allow_images: bool,
        }
        fn app(p: P) -> Element {
            rsx! {
                Markdown { source: p.source, streaming: p.streaming, allow_images: p.allow_images }
            }
        }
        let mut dom = VirtualDom::new_with_props(
            app,
            P {
                source: source.to_string(),
                streaming,
                allow_images,
            },
        );
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn link_policy_allowlist() {
        for ok in [
            "https://example.com/a?b=c#d",
            "http://example.com",
            "HTTPS://EXAMPLE.COM",
            "mailto:a@example.com",
            "tel:+15551234",
            "/docs/page",
            "page.html",
            "#section",
            "?q=1",
            "//cdn.example.com/x",
        ] {
            assert!(safe_href(ok).is_some(), "{ok} should be allowed");
        }
        for bad in [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            " javascript:alert(1)",
            "java\tscript:alert(1)",
            "java\nscript:alert(1)",
            "\u{1}javascript:alert(1)",
            "vbscript:msgbox(1)",
            "data:text/html,<script>alert(1)</script>",
            "file:///etc/passwd",
            "blob:https://example.com/x",
            "ftp://example.com",
            "http:example.com",
            "https://",
            "http:///evil",
            "/\\evil.example",
            "\\\\evil.example",
            "///evil.example",
            "mailto:",
            "",
            "   ",
            "a b",
        ] {
            assert!(safe_href(bad).is_none(), "{bad:?} should be refused");
        }
        assert!(safe_href("https://a.com").unwrap().is_external());
        assert!(safe_href("//a.com").unwrap().is_external());
        assert!(!safe_href("mailto:a@b.c").unwrap().is_external());
        assert!(!safe_href("/x").unwrap().is_external());
    }

    #[test]
    fn heal_cases() {
        let cases = [
            ("Hello **wor", "Hello **wor**"),
            ("Hello *wor", "Hello *wor*"),
            ("Hello _wor", "Hello _wor_"),
            ("a `code", "a `code`"),
            ("a ``co`de", "a ``co`de``"),
            ("text `", "text "),
            ("a ``", "a "),
            ("~~strike", "~~strike~~"),
            ("**bold *nested", "**bold *nested***"),
            ("Hello **", "Hello "),
            ("**bold word ", "**bold word**"),
            ("**bold** and *it", "**bold** and *it*"),
            ("see [docs](https://exa", "see docs"),
            ("see [docs]( ", "see docs"),
            ("see [docs", "see docs"),
            ("see ![alt](https://exa", "see alt"),
            (
                "see [docs](https://example.com)",
                "see [docs](https://example.com)",
            ),
            ("snake_case_word", "snake_case_word"),
            ("2 * 3 = 6", "2 * 3 = 6"),
            ("- item\n- **bold", "- item\n- **bold**"),
            ("para one **x\n\npara two", "para one **x\n\npara two"),
            ("```rust\nfn main() {", "```rust\nfn main() {"),
            ("```rust\nfn main() {}\n``", "```rust\nfn main() {}\n"),
            (
                "```rust\nlet a = **b;\n```\n\nafter *x",
                "```rust\nlet a = **b;\n```\n\nafter *x*",
            ),
            (
                "~~~\ncode *x\n~~~\nthen **y",
                "~~~\ncode *x\n~~~\nthen **y**",
            ),
            ("a \\*not emphasis", "a \\*not emphasis"),
            ("", ""),
        ];
        for (input, want) in cases {
            assert_eq!(heal_markdown(input), want, "heal({input:?})");
        }
    }

    /// Total and idempotent on every prefix of a nasty document, with multibyte characters.
    #[test]
    fn heal_is_total_and_idempotent_on_every_prefix() {
        let doc = "# Title *é*\n\nSome **bold `code` and *nested_ _text_** with [a link](https://example.com/x_y) \
and ![img](https://e.com/i.png) plus ~~gone~~ and 日本語 **語\n\n- [ ] task `a\n- [x] done\n\n```rust\nfn x() { \"**\" }\n```\n\n\
| a | b |\n|---|---|\n| `1` | **2 |\n\n> quote [x](y\n\nhttps://example.com/a_b_c, www.example.org). end";
        for (i, _) in doc.char_indices().chain([(doc.len(), ' ')]) {
            let prefix = &doc[..i];
            let once = heal_markdown(prefix);
            assert_eq!(heal_markdown(&once), once, "not idempotent on {prefix:?}");
            // And the healed prefix always parses and renders.
            let _ = build_tree(&once);
        }
    }

    #[test]
    fn raw_html_is_inert_text() {
        for evil in [
            "<script>alert(1)</script>",
            "<img src=x onerror=alert(1)>",
            "text <b onclick=alert(1)>x</b> text",
            "<iframe src=\"javascript:alert(1)\"></iframe>",
            "<a href=\"javascript:alert(1)\">x</a>",
            "<svg onload=alert(1)>",
        ] {
            let html = render(evil, false, true);
            for needle in [
                "<script",
                "<img",
                "<iframe",
                "<svg",
                "<b ",
                "<a ",
                "onerror=\"",
                "href=\"javascript",
            ] {
                assert!(
                    !html.contains(needle),
                    "{needle} reached the DOM for {evil:?}: {html}"
                );
            }
            // The SSR renderer escapes `<` numerically (`&#60;`); either spelling is escaped text.
            assert!(
                html.contains("&#60;") || html.contains("&lt;"),
                "the markup must appear escaped: {html}"
            );
        }
    }

    #[test]
    fn links_render_per_policy() {
        let html = render(
            "[a](https://example.com) [b](javascript:alert(1)) [c](/rel) [d](mailto:x@y.z)",
            false,
            false,
        );
        assert!(html.contains("href=\"https://example.com\""));
        assert!(html.contains("target=\"_blank\""));
        assert!(html.contains("rel=\"noopener noreferrer\""));
        assert!(!html.contains("javascript"), "{html}");
        assert!(html.contains("data-blocked=\"link\""));
        assert!(html.contains("href=\"/rel\""));
        assert_eq!(
            html.matches("target=\"_blank\"").count(),
            1,
            "only the web link opens a tab: {html}"
        );
        // An entity cannot smuggle a scheme past the check: the parser decodes it first.
        let html = render(
            "[x](javascript&colon;alert(1)) [y](&#106;avascript:alert(1))",
            false,
            false,
        );
        assert!(!html.contains("href"), "{html}");
    }

    #[test]
    fn images_are_off_by_default() {
        let src = "![the alt](https://evil.example/?q=secret)";
        let off = render(src, false, false);
        assert!(
            !off.contains("<img") && !off.contains("evil.example"),
            "{off}"
        );
        assert!(off.contains("the alt") && off.contains("data-blocked=\"image\""));
        let on = render(src, false, true);
        assert!(
            on.contains("<img") && on.contains("referrerpolicy=\"no-referrer\""),
            "{on}"
        );
        let bad = render("![x](javascript:alert(1)) ![y](mailto:a@b.c)", false, true);
        assert!(!bad.contains("<img"), "{bad}");
    }

    #[test]
    fn bare_urls_become_links() {
        let html = render(
            "See https://example.com/path, and (www.example.org/x). Not a link: ahttps://x.y.",
            false,
            false,
        );
        assert!(html.contains("href=\"https://example.com/path\""), "{html}");
        assert!(html.contains("href=\"http://www.example.org/x\""), "{html}");
        assert!(!html.contains("ahttps://x.y\""), "{html}");
        // Not inside code, and not doubled inside an explicit link.
        let html = render(
            "`https://a.example` [https://b.example](https://b.example)",
            false,
            false,
        );
        assert_eq!(html.matches("<a ").count(), 1, "{html}");
    }

    #[test]
    fn code_block_and_language_class() {
        let html = render(
            "```rust\nfn main() {}\n```\n\n```we ird<>\nx\n```",
            false,
            false,
        );
        assert!(html.contains("language-rust"), "{html}");
        assert!(!html.contains("ird<"), "{html}");
        assert!(html.contains("fn main() {}"));
    }

    #[test]
    fn structure_renders() {
        let html = render(
            "# H\n\n1. one\n2. two\n\n- [x] done\n- [ ] todo\n\n| a | b |\n|:-|-:|\n| 1 | 2 |\n\n> q\n\n---\n\n~~s~~",
            false,
            false,
        );
        for needle in [
            "<h1",
            "<ol",
            "type=\"checkbox\"",
            "<table",
            "scope=\"col\"",
            "data-align=\"end\"",
            "<blockquote",
            "<hr",
            "<del",
        ] {
            assert!(html.contains(needle), "missing {needle}: {html}");
        }
    }

    #[test]
    fn streaming_prop_heals() {
        let html = render("Hello **wor", true, false);
        assert!(html.contains("<strong>wor</strong>"), "{html}");
        let html = render("Hello **wor", false, false);
        assert!(!html.contains("<strong>"), "{html}");
    }
}
