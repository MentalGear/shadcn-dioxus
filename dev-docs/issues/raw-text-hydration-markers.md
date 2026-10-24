# Dynamic children inside raw-text elements break SSR hydration (backlog row 97)

Measured 2026-10-23 against dioxus 0.7.9 (the lockfile's `dioxus-core`, `dioxus-ssr`, `dioxus-web`,
`dioxus-interpreter-js`) and Chromium 1243.

## What row 97 claimed, and why it was wrong

Row 97 said: "a second, un-fragmented rsx sibling renders on the first pass and is silently dropped by the
next one", from `CarouselContent`'s scoped `<style>` tag (commit `67f05223`), and that wrapping the siblings in
an explicit `Fragment` fixed it. Two measurements say that bug does not exist:

1. **The original regression test passes without the `Fragment`.** At `67f05223`, with the `Fragment` wrapper
   removed from `CarouselContent` and nothing else changed,
   `cargo test -p dioxus-primitives --lib no_skip_style` runs both of that commit's tests green, including
   `no_skip_style_tag_survives_the_post_effect_re_render` (`rebuild_in_place`, four `render_immediate` passes,
   then `dioxus_ssr::render`). So the test never failed on the shape it was written to pin.
2. **A VirtualDom repro of every multi-root shape keeps the sibling.** Two static roots; a root with
   `{children}`; a component root then an element; `{children}` then an element; a spread root then a `style`
   with dynamic text; a conditional sibling; a parent re-rendering a two-root child; and the exact carousel
   shape (`style { {css} }` then `div { id, style, ..rest_attrs, {children} }`, children registering through
   effects). All seven render the second root on the first pass and after the effects re-render.

## What actually happens: hydration markers inside raw-text elements

`dioxus-ssr` writes every dynamic text node as `<!--node-idN-->text<!--#-->` so the client can find it while
hydrating (`dioxus-interpreter-js` `src/ts/core.ts`, the `node-id` branch of the comment walker). Inside an
HTML raw-text element (`style`, `script`, `xmp`, `iframe`, `noembed`, `noframes`, and `noscript` with scripting
on) or an escapable raw-text element (`textarea`, `title`) the parser does not make comments: those markers
stay literal characters of the element's text.

A fullstack app served by `dx serve` (SSR, then hydration), each case a component whose effect re-renders it
twice, read in Chromium after hydration settled:

| case | element text after hydration | `sheet.cssRules` | rule applies |
|---|---|---|---|
| two bare roots, `style { {css} }` first | `<!--node-id2-->#bare { color: … }<!--#-->` | 1 (the garbage rule) | no |
| the same inside `Fragment { }` | `<!--node-id6-->#frag { color: … }<!--#-->` | 1 (the garbage rule) | no |
| `style { {css} }` as a child of the `div`, one root | `<!--node-id14-->#single { … }<!--#-->` | 1 (the garbage rule) | no |
| static `style { "#static-style {{ … }}" }` | `#static-style { … }` | 1 | **yes** |
| `style { dangerous_inner_html: css }` | `#inner { … }` | 1 | **yes** |
| `textarea { "{v}" }` | value `<!--node-id24-->val-area<!--#-->` | | |
| `textarea { value: "{v}" }` | value empty before hydration, `val-…-2` after | | |
| `span { {t} }` beside a div (control) | `dyn-spansib` | | |

In CSS, `<!--` and `-->` are CDO/CDC tokens a stylesheet's top level skips, but `node-idN` right after the
first one starts a qualified rule whose prelude runs on to the real selector and takes its `{…}` block as its
own. The sheet ends up with one rule nobody matches. Pure CSR (`dx serve --web`) emits no markers, which is
why it looked right in a dev loop.

So the carousel symptom (`scroll-snap-stop: normal`) had one cause, the markers, and the `Fragment` was not a
fix for anything: bare roots and `Fragment` behave the same in every row of the table. Row 96's correction
(2026-09-20) found the markers; this run shows they were the whole story. The sibling position plays no part
(a lone `style` child of a `div` fails the same way).

## The construction

`scripts/check-attr-spread-collision.sh`'s analyzer (`scripts/attr-spread-collision-checker`, which already
parses every `rsx!` body with `dioxus-rsx`) reports a third finding kind, `raw-text-dynamic-child`: an element
named in `RAW_TEXT_ELEMENTS`, outside `svg`/`math` (where the same names are foreign elements whose content is
parsed, e.g. `chart.rs`' `svg { title { "{label}" } }`), with any child other than one static text literal:
an interpolated string, a `{expr}` block, a component, an `if`/`for`, or a nested element. It has no baseline:
the sweep of `primitives/src`, `preview/src` and `templates` found 0 sites (every preview demo `style` is a
static `r#"…"#` literal; `docs_nav.rs`' `script` uses `dangerous_inner_html`), so any finding fails the gate.
The fix it points to is static text, or `dangerous_inner_html` for a string that must be computed.

Not covered: an element built by name at runtime, or raw-text content assembled outside `rsx!` (neither exists
here). Row 97's "fix by construction" is this check; the multi-root claim needs no construction because the
behaviour it describes does not occur.
