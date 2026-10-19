The CodeBlock component frames a snippet: an optional header (filename, actions), the code in a scrollable block, an optional footer. It owns the frame and the copy button only. It bundles no highlighter, so none of its dependencies reach your project unless you add one.

## Component Structure

```rust
CodeBlock {
    // Available variants: Default, Destructive. Sizes: Default, Sm
    variant: CodeBlockVariant::Default,
    size: CodeBlockSize::Default,

    CodeBlockHeader {
        CodeBlockTitle { "app/api/chat/route.ts" }
        CodeBlockActions {
            CodeBlockCopyButton { value: CODE }
        }
    }
    // Plain text, split into lines for you:
    CodeBlockContent { code: CODE, line_numbers: true }
    CodeBlockFooter { "Exited with code 0" }
}
```

- `CodeBlockActions` as a direct child of `CodeBlock` (no header) floats over the top corner, visible on hover or focus and always on touch screens.
- `wrap` wraps long lines; by default they scroll sideways. `start_line` offsets the line numbers for a snippet from the middle of a file.
- Line numbers are CSS counters: they are not selectable, not copied and not announced.

## Bring a highlighter

Render one `CodeBlockLine` per line, with your highlighter's spans as its children, inside `CodeBlockContent`. Give an empty line a `"\n"` child so a copied selection keeps its blank lines. `CodeBlockLine`'s `variant` marks a diff (`Added`, `Removed`) or a call-out (`Highlighted`); added and removed lines also get a `+` / `−` in the gutter, so the cue is not colour alone.

```rust
CodeBlockContent { line_numbers: true,
    for (i, line) in highlighted.iter().enumerate() {
        CodeBlockLine { key: "{i}", {line.clone()} }
    }
}
```

The copy button copies the `value` you give it (the verbatim code), never the highlighted markup.

`Markdown`'s `highlight` slot renders inside Markdown's own `<pre>`, and `CodeBlockContent` is a `<pre>` too, so the two do not nest. To show a framed block with a header inside a reply, split the reply at its fences and render the prose with `Markdown` and the code with `CodeBlock` (the Code Block Chat demo on Message does this).

## Accessibility

- The copy button is a real `<button>` whose accessible name flips from "Copy" to "Copied" for about two seconds; a visually hidden `aria-live="polite"` region announces the change.
- The code region is focusable (`tabindex="0"`) so a keyboard user can scroll it; focus shows the theme's focus ring on the whole block.
- Code is always `dir="ltr"`, also in an RTL page.
- The destructive variant uses the theme's clamped destructive text colour; say what failed in the title, not only in the colour.
