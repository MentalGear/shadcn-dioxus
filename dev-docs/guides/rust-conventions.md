# Rust conventions

Six conventions adopted from the Google Rust style guide (owner 2026-10-21, [row 355](../backlog/355-adopt-google-rust-style-guide-items.md)).
They apply to `primitives/`, `cli/` and `preview/`. The lint-enforced items of the same row (the workspace lint table,
`unwrap_used`, `return_self_not_must_use`, `#[expect]` with a reason) are lints, not prose, and live in `Cargo.toml` and
`clippy.toml` once they land; the gates ([`gates.md`](../gates.md)) enforce the rest of the repo's source rules.

1. **An assert message says why, not what.** `assert!(idx < len, "idx {idx} out of range for {len} rows")` names the
   state that broke; `assert!(idx < len, "index too large")` repeats the condition. The same goes for `expect("...")`: it
   states the invariant that makes the call safe (`expect("the root context is provided by the Dialog root")`), never
   "failed to get context".
2. **An enum over a bare `bool` parameter.** `render(row, true, false)` reads as nothing at the call site; `render(row,
   Selection::Selected, Focus::None)` reads as itself, and a third state later is a new variant, not a new flag. A
   `bool` field of a props struct that names its own meaning (`disabled`, `open`) is fine; a positional `bool` argument
   is not.
3. **Accessors are `foo()`, `set_foo()` and `foo_mut()`.** No `get_` prefix on a getter (`size()`, not `get_size()`),
   `set_` for a setter, `_mut` for the `&mut` borrow. A hook that returns a signal or a handle is named for what it
   returns (`use_direction`), not as an accessor.
4. **An `Ext` suffix for an extension trait.** A trait that only adds methods to a type you do not own is `FooExt`
   (a `StrExt` that adds methods to `str`), so the reader knows it is not a role other types implement.
5. **No trait only to organize code.** A trait exists because two or more types are used through it. A trait with one
   implementor, or one that groups free functions, is a module and some functions: modules organize, traits abstract.
6. **A type annotation over a turbofish.** `let rows: Vec<Row> = iter.collect();` over `iter.collect::<Vec<_>>()`: the
   annotation sits where a reader looks for the type, and the compiler's error points at the binding. A turbofish stays
   where there is no binding to annotate (a function argument, a chained call that needs the type to continue).

When a convention and a gate disagree, the gate wins and this page is fixed. A new convention that a script can check
becomes a gate (its header, `run-gates.sh`, `scripts/gates-doc.py --write`), not a paragraph here.
