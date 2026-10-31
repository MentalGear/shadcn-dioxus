Markdown renders model output as prose. It parses the text with `pulldown-cmark`, folds the events into a small allowlisted tree, and renders that tree to Dioxus elements, so unsafe HTML and unsafe links cannot reach the page by construction. It is an optional component: `message`, `bubble` and `message_scroller` do not depend on it, and its parser crate is only added to your project if you run `shadcn-dioxus add markdown`.

## Component Structure

```rust
// A finished message.
Markdown { source: message_text }

// A reply that is still arriving: heal the unfinished tail on every chunk.
Markdown { source: partial_text(), streaming: is_generating() }

// Inside a chat bubble.
Bubble { variant: BubbleVariant::Ghost,
    BubbleContent {
        Markdown { source: reply_text }
    }
}
```

| Prop | Default | |
| --- | --- | --- |
| `source` | required | The Markdown text. |
| `streaming` | `false` | The text is still being generated: heal an unfinished tail before parsing. |
| `allow_images` | `false` | Load images from the text. Opt in only for text you trust. |
| `highlight` | none | `Callback<CodeBlock, Element>`: renders one code block. |

## Safety

Safe by construction, not by filtering: the renderer reads only the allowlisted tree, which has no raw-HTML variant and stores a link target only as a `SafeHref` that passed the link policy.

- **Raw HTML is text.** `<script>`, `<img onerror=...>` and every other tag appear as literal, escaped text. Nothing in the component uses `dangerous_inner_html`.
- **Links** are `http`, `https`, `mailto`, `tel` or relative only. `javascript:`, `data:`, `vbscript:`, `file:`, unknown schemes, and targets containing whitespace, control characters or a backslash (`java<TAB>script:`, `/\evil.example`) lose their target and keep their label. Entities are decoded by the parser before the check, so `javascript&colon;` is refused too.
- **External links** (`http`, `https`, `//host`) open in a new tab with `rel="noopener noreferrer"`. `mailto:`, `tel:` and relative links open normally.
- **Images are off by default.** A model can encode data in the URL of an image the browser loads on its own, so the alt text is shown and nothing is fetched. Set `allow_images: true` to load `http`, `https` and relative images, lazily and with `referrerpolicy="no-referrer"`.
- **No injection options.** Heading ids and classes, metadata blocks, footnotes, math and smart punctuation are not enabled. A code fence's language becomes a class name only if it is a short plain identifier.

## Streaming

Set `streaming: true` while text arrives. `heal_markdown` repairs only the last paragraph before parsing: an unclosed `` ` `` or fence tail, unclosed `**`, `*`, `_` and `~~` get closers, and `[label](https://exa` shows just its label until the target is complete. It is total, idempotent and tested on every prefix of a document. Each chunk re-parses the whole text, which is fine for chat-sized messages. Pair it with `MessageScroller` to keep the live edge in view.

Bare URLs (`https://example.com`, `www.example.org`) become links through the component's own pass, since `pulldown-cmark` has no GFM autolink literal. Math is not supported.

## Code blocks

A block is a plain `<pre><code class="language-rust">`. To highlight, pass `highlight`; it receives `CodeBlock { language, code }` and returns the contents of the `<pre>`:

```rust
Markdown {
    source,
    highlight: Callback::new(|block: CodeBlock| rsx! {
        MyHighlighter { language: block.language, code: block.code }
    }),
}
```

No highlighter ships with the component, so none of its dependencies reach your project unless you add one.

## Dependencies

`shadcn-dioxus add markdown` runs `cargo add pulldown-cmark@=0.13.4 --no-default-features`: one extra crate in a typical Dioxus tree (`unicase`; `bitflags` and `memchr` are already there). The pin is exact because 0.13.4 carries a panic fix for the tight-paragraph case that a wasm app cannot recover from. Relax it in your `Cargo.toml` (`~0.13.4`) if another dependency needs a newer 0.13.x.

## Accessibility

- Prose uses real headings, lists, tables with column headers, and blockquotes. Heading levels are the text's own; keep them in order in your page.
- A task-list checkbox is disabled and labelled "Completed" or "Not completed".
- A code block and a table that can scroll horizontally take focus so the keyboard can scroll them; a wide table is a labelled region.
- Link colour is the theme's primary ink and links are underlined, so colour is not the only cue.
- Set the monospace face for code with `--dx-font-mono` on an ancestor; without it code uses the surrounding font.
