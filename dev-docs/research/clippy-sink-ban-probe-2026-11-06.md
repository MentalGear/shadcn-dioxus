# Clippy sink-ban probe (row 444, item 0.1)

Question: can `clippy.toml` `disallowed-methods` carry the `document::eval` sink ban and the `DataTransfer` setter ban,
with a per-site `#[expect(clippy::disallowed_methods, reason = "...")]`? Measured in a scratch crate outside the repo
(dioxus 0.7.9 from the repo's `Cargo.lock`, `dioxus-attributes` as a path dependency, cargo 1.97.0), clippy run as
`cargo clippy` and `cargo clippy --features web`, each also with `--all-targets`. Both arms gave identical hits.

## `clippy.toml` entries that fired

```toml
disallowed-methods = [
  { path = "dioxus::document::eval", reason = "..." },
  { path = "dioxus_document::Document::eval", reason = "..." },
  { path = "dioxus_html::DataTransfer::set_data", reason = "..." },
  { path = "dioxus_html::DataTransfer::set_drop_effect", reason = "..." },
  { path = "dioxus_html::DataTransfer::set_effect_allowed", reason = "..." },
  { path = "dioxus_html::DataTransfer::clear_data", reason = "..." },
  { path = "dioxus::document::Script", reason = "..." },  # optional, see case 3
]
```

The re-export spelling (`dioxus::document::eval`) and the defining spelling (`dioxus_document::eval`) each work alone;
the lint message prints the spelling configured. `dioxus_html::...` resolves without a direct `dioxus-html`
dependency (a transitive crate is enough).

## Cases

| # | Site | Fires | Notes |
|---|------|-------|-------|
| 1 | direct `document::eval(..)`, `dioxus::document::eval(..)`, and a bare `use dioxus::document::eval; eval(..)` | yes (all three) | the repo's `checkbox.rs` / `default_checked.rs` use the bare form |
| 2 | inside an rsx event handler (`onscroll: move \|_\| { document::eval(..); }`, and the expression-bodied `onclick: move \|_\| document::eval(..).send(..)`) | yes | also inside `attributes!(tr { ondragover: .. })` (dioxus-attributes) and in a component-prop closure such as `on_select:` (same expansion shape) |
| 3 | `document::Script {}` / `Style {}` / `Link {}` in rsx | not under the eval entries (Script's own `eval` runs inside `dioxus-document`, which is not linted); with a `dioxus::document::Script` entry it fires at the rsx element, in both feature arms | the repo has one `document::Script` site (`primitives/src/lib.rs:141`, `cfg(not(feature = "web"))`) |
| 4 | function pointer `let f = document::eval;` | yes, at the path expression; the later call `f("..")` does not fire | an expect is needed on the `let f` line only |
| 5 | `DataTransfer::{set_data, set_drop_effect, set_effect_allowed, clear_data}` on `event.data_transfer()` | yes, all four, in rsx closures and in `attributes!` closures | |
| 6 | `Document::eval` trait-method call (`d.eval(..)` on `Rc<dyn Document>`, `self.inner.eval(..)`) | yes with the `dioxus_document::Document::eval` entry; the `fn eval` definition inside an `impl Document` does not fire | `eager_head.rs` needs this entry |

## Where an expect can sit

Every placement below was run and the expectation was fulfilled (no `unfulfilled_lint_expectations`).

| Site kind | Placement | Exact text |
|-----------|-----------|------------|
| `let x = document::eval(..);` / `let _ = ..;` / `_ = ..;` / `document::eval(..);` statement | on the statement | `#[expect(clippy::disallowed_methods, reason = "...")]` on the line above |
| tail expression of a block, `document::eval(..)` with no `;` | on the tail expression itself (accepted), or a fn-level expect | same attribute above the expression |
| inside a closure in rsx (`onscroll: move \|_\| { .. }`) | (a) on the inner statement; (b) **on the closure, in the attribute value position**: `onscroll: #[expect(clippy::disallowed_methods, reason = "...")] move \|_\| { .. },`; one expect covers every call inside the closure (inferred from lint scoping, tested with one call) | (b) is the nearest placement to the rsx attribute: an rsx attribute itself takes no attribute, its value expression does |
| closure in `attributes!(tr { ondragover: .. })` | same two placements, (b) tested | |
| `use_effect(#[expect(..)] move \|\| { document::eval(..); })` | on the closure argument | |
| `let handler = #[expect] move \|..\| { eval }` | on the `let` | |
| whole `rsx!` | `#[expect(..)] let el = rsx! { .. }; el`, or on the fn | works, but covers the whole element tree: wider than needed |
| function pointer `let f = document::eval;` | on the `let` | |
| `Script {}` in rsx | on the enclosing fn, or on `let el = rsx! { .. };` | a child element in rsx cannot carry an attribute |
| `impl Document` method body `self.inner.eval(js)` | on the statement, or on the method | |

`unfulfilled_lint_expectations` fires when the call is removed: a probe `#[expect(..)] let _ = 1;` and an expect above a
call that is not a hit (the call `f("..")` of a fn pointer) both gave "this lint expectation is unfulfilled". With the
repo's `[workspace.lints.clippy]` and `-D warnings`, a stale expect is an error. An expect inside a `cfg`-removed arm
disappears with the arm, so no cross-arm unfulfilled case arises from cfg-gated sites.

Limits found: (i) `disallowed-methods` flags the path expression, not the type, so a fn pointer is flagged where it is
taken and not where it is called; (ii) the lint is per call site, so a macro that expands to `eval` inside a crate
without the ban (`dioxus-document` itself) is not seen; (iii) `disallowed-methods` has no test exemption (unlike
`allow-unwrap-in-tests`), so `cfg(test)` sites need an expect too (`eager_head.rs:170`).

## Counts today (`primitives`, `preview`, `cli`, `labs`, `test-harness`, `docs`, `templates`)

Non-comment calls of `eval(`: **69** (`primitives` 40, `preview` 29; `cli`, `labs`, `test-harness`, `docs`, `templates`
have none).

| Kind | Sites | Expect placement |
|------|------:|------------------|
| `document::eval` / `dioxus::document::eval` outside rsx, as `let x =` or `let _ =` | 37 | statement |
| same, as `_ = ..;` (`preview/src/theme.rs:146,180`) | 2 | statement |
| same, as an expression statement `document::eval(..);` (one is a tail expression: `primitives/src/bridge/transport.rs:189`) | 15 | statement; the tail one takes the statement-position attribute or a fn-level expect |
| inside an rsx event-handler closure, bound (`let`): `chart.rs:693`, `form/component.rs:411,644,678`, `code_block/component.rs:198`, `photo_wall/mod.rs:419,427`, `charts_gallery.rs:535` | 8 | inner statement (or closure value) |
| inside an rsx event-handler closure, expression statement: `top_layer/component.rs:745` | 1 | inner statement (or closure value) |
| inside a component-prop closure in rsx (`docs_nav.rs:247`, `on_select:`) | 1 | inner statement |
| bare imported `eval(..)` (`primitives/src/checkbox.rs:319`, `default_checked.rs:38`) | 2 | statement |
| `Document::eval` trait-method calls (`preview/src/eager_head.rs:87`, `:103`, `:170`; `:170` is inside `cfg(test)`) | 3 | statement or method |
| **Total eval expects** | **69** | 64 `document::eval` + 2 bare + 3 trait; fewer if fn-level expects are used |

`document::Script`: 1 rsx site (`primitives/src/lib.rs:141`), needs a fn-level expect only if a `Script` entry is added.
`eager_head.rs`: 1 file, 3 `Document::eval` calls, plus the `impl Document for EagerHeadDocument` and the test
`Platform` impl, whose `fn eval` definitions do not fire.

`DataTransfer` setters: **14 calls in 8 handlers**, all inside rsx or `attributes!` closures
(`primitives/src/drag_and_drop_list.rs` 5 calls / 3 handlers: `:610`, `:862-865`, `:874`;
`preview/src/components/data_table/reorder.rs` 4 / 2: `:288`, `:365-368`;
`data_table/component.rs:537` 1 / 1; `virtual_list/variants/reorder/mod.rs` 4 / 2: `:146-149`, `:165`).
Only `set_drop_effect`, `set_effect_allowed` and `set_data` occur; `clear_data` has no site. Placement: closure value
(8 expects) or per call (14).
