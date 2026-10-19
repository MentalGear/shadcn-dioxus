---
id: 80
title: "CSS hot reload root cause: a compile-time embed, not the folder location"
status: "Landed 2026-09-18/19 (batch 2); supersedes row 72's \"move stylesheets\" bullet and row 76's number"
size: "M"
area: theme
updated: 2026-09-18
---

# Row 80: CSS hot reload root cause: a compile-time embed, not the folder location

## Item

CSS hot reload root cause: a compile-time embed, not the folder location

## Status / trigger

Landed 2026-09-18/19 (batch 2); supersedes row 72's "move stylesheets" bullet and row 76's number

## Size

M

## Detail

**`238c8f4`.** Root cause: `dioxus_code::code!()` expands to `include_str!`, so a themed component's CSS text is compiled directly into the `preview` binary and lands in rustc's own dep-info; `dx`'s file watcher (`runner.rs::handle_file_change`) checks a changed non-`.rs` file against that dep-info list and, on a match, forces `needs_full_rebuild`, overriding that same file's own `asset!()` hot-reload path — the CSS is both compiled in and asset-served, and the compiled-in copy wins the watcher's decision every time. `build.rs`'s blanket `rerun-if-changed=src/components` was a compounding second cause, proven separately: narrowing it alone still produced a full rebuild, so it was not the sole mechanism. The same defect exists for `THEME_CSS` (`preview/assets/dx-components-theme.css`), which disproves row 76's original "CSS under `src/` is not cheap, `preview/assets/` CSS hot-reloads" theory — the mechanism is the compile-time embed, not which directory the file lives in. **Construction:** the `code!()` embed stays for release/SSG builds (`#[cfg(not(debug_assertions))]`, byte-identical output — not verified live in a browser, compile+lint only), so nothing changes for the shipped artifact; debug builds instead carry the `asset!()` URL and fetch-plus-syntax-highlight the CSS at runtime (`preview/src/main.rs`'s new `LazyCssCodeBlock`), so the watcher's dep-info list no longer contains the CSS text at all and the file's own `asset!()` hot-reload path is free to win. Measured: a `kbd/style.css` edit went from a 45.9s full rebuild to a 0.8s hot-reload, confirmed against a fresh, never-before-opened page load (not just an already-open tab, `dev-loop.md`'s own Trap 4). Checks: `fmt`, `check`/`clippy` in both debug and release, `cargo test --workspace` 23/23 (this component), all guards, stylelint, `preview.spec.ts` +1 ("style tab") 3/3. **Supersedes, in place:** row 72's "move component stylesheets so CSS edits hot-reload" bullet (the diagnosis there was directionally right about the symptom but named the wrong variable — moving files would not have fixed it, since `THEME_CSS` already lived in `preview/assets/` and had the identical defect) and row 76's "~28s" rebuild number (now historical: the same edit class is ~0.8s post-fix). `dev-loop.md` already carries this lane's own full write-up in its "CSS root cause, corrected" section — see there rather than duplicating it here
