---
id: 429
title: "One gate kit: a shared CSS walker and one allowlist engine for the check scripts"
status: "LANDED (lane/gate-kit-429). Owner approved 2026-10-25 (row 365 sweep, P4)."
size: "M"
area: tooling
updated: 2026-10-25
---

# Row 429: One gate kit, a shared CSS walker and one allowlist engine

## Item

Eighteen `scripts/check-*.sh` heredocs each embed their own comment stripper, brace walker or selector splitter, and three of them already read the same input differently (quoted-string escapes, newline-preserving comment strip, rules nested in `@media`). About 40 scripts hand-implement "a reason is required, a stale entry fails, `--list` prints". Construction: `scripts/lib/css.py` (one quote-, paren- and escape-aware scanner returning `rules(css)`) and `scripts/lib/allow.py` (`Allowlist(entries)`, mandatory reason, `used` tracking, stale report). Two gates that read one stylesheet then read it one way by construction.

## Status / trigger

LANDED (lane/gate-kit-429). Owner approved 2026-10-25 (row 365 sweep, P4).

## Size

M

## Detail

Source: [final sweep note, P4](../../research/final-sweep-2026-10-25.md). Scripts only, no product code. One lane owns `scripts/check-*.sh` while it runs.

- **Duplicated** (`grep -lE "^def (strip_comments|split_top|...)" scripts/check-*.sh`): `check-variant-scope.sh:108-141` (`style_rules`), `check-cross-component-overrides.sh:112-173` (`parse_rules`), `check-popup-boundary.sh:113-160`, `check-press-under.sh:46-62` (`rules`), `check-popup-root-reset.sh:107-117` (`find_rule`), `check-infinite-animations.sh:69-110` (`blocks`/`walk`), `check-hover-media.sh:73,188`, `check-component-root-reset.sh:150`, `check-demo-wrapper-width.sh:86,192`, `check-theme-preset-axes.sh:43`, `check-focus-outline-reset.sh:59`, `check-scroll-behavior.sh:84`, `check-theme-baseline.sh:143`, `check-cursor-baseline.sh:93`. `split_top` is defined in 10 scripts, `strip_comments` in 10; 18 scripts, 5,031 lines.
- **Drift (verified by reading):** `cross-component-overrides.sh:121-130` and `popup-boundary.sh:121-130` handle a backslash in quoted strings, `variant-scope.sh:115-123` does not (a `\"` in `content:` ends the quote early). `press-under.sh:46-47` strips comments without keeping newlines, so its reported lines are off. `popup-root-reset.sh:107-117` does not see a rule nested in `@media`/`@supports`. None is a known false negative today.
- **Subsumes:** the three drifts, roughly 1,000 duplicated lines, and the next gate's boilerplate. **Does not subsume:** the Rust-source scanners (`comment-scan.py` is already shared; `wrapper-props` and `attr-spread-collision-checker` are `syn` tools) and the shell-only gates.
- **Gate:** `check-gate-kit.sh` with fixtures for escapes, nested at-rules, comments containing newlines and `url(data:...;...{)`, plus a grep rule that no `check-*.sh` heredoc defines `strip_comments` or `split_top`. `check-gate-reuse.sh` and `cargo-inputs-hash.sh` add the lib to their declared inputs.
- **Risk:** a migration can loosen a gate silently. Each migrated gate keeps its mutation self-test as the oracle, and `run-gates.sh --list` output is captured before and after.

## Outcome

- **Built:** `scripts/lib/css.py` (one scanner: quoted strings with backslash escapes, unquoted `url(...)` taken whole, parenthesis depth, comments blanked with their newlines kept so line numbers hold, nested at-rules with the `at` chain, nested rules, unbalanced braces reported as `problems`; `strings=False` for non-CSS text and `fragment=True` for CSS inside a Rust raw string or HTML) and `scripts/lib/allow.py` (`Allowlist`: fields plus a mandatory reason, `find`/`find_all` mark use, `problems()` fails a blank reason, a duplicate and a stale entry, `listing()` for `--list`).
- **Gate:** `scripts/check-gate-kit.sh` (in `run-gates.sh` after `check-gate-reuse`): fixtures for escapes, nested at-rules, newline comments and `url(data:...{)`, 26 library mutations that must turn red, the grep rule (no `check-*.sh` defines `strip_comments`, `split_top` or the other banned walkers), the rule that a gate importing the kit also inserts `scripts/lib` on its path, the stdlib-only rule, and the cargo-reuse rule below.
- **Migrated (28 gates, -999/+431 lines in the gates, +879 in the lib and its gate):** variant-scope, press-under, popup-boundary, cross-component-overrides, popup-root-reset, component-root-reset, theme-baseline, cursor-baseline, focus-outline-reset, scroll-behavior, hover-media, infinite-animations, anchored-keyframes, duplicate-keyframes, css-vars-defined, css-literals, coarse-sizing, css-logical-properties, css-state-recipes, hover-mix-space, destructive-text, component-fonts, theme-preset-axes, dx-class-prefix, demo-wrapper-width, demo-remote-assets, spec-at-rest, motion-gating. `run-gates.sh --list` differs only by the added line.
- **Verification:** baseline of 82 fast gates (all PASS) captured first; every migrated gate was run old against new on the real tree and on mutated trees (descendant combinators, nested at-rules, escaped quote, `url(data:..{)`, comments with newlines, stale entries). Verdict and output are identical on the real tree for all but `check-dx-class-prefix`, which keeps its verdict and warnings with corrected line numbers. After: 83 fast gates, all PASS. The in-gate mutation self-tests (variant-scope, popup-root-reset, component-root-reset, cursor-baseline, theme-preset-axes) stay red on every mutation.
- **What the one reader exposed (none changes a verdict on the real tree, every parse reports 0 problems):** variant-scope missed a rule after an escaped quote; press-under missed a rule after `url(data:...{)` and its lines were off; dx-class-prefix reported wrong lines and skipped the rule after an `@import ...;` statement; hover-media was red for the wrong reason ("unbalanced braces") on an escaped quote or `url(data:)`; press-under, css-state-recipes, hover-mix-space and focus-outline-reset only reported a stale allowlist entry and now fail it (none is stale today). popup-root-reset's claimed blindness to a rule inside `@media` is not borne out as a false negative on the tree; it now reads nested rules anyway.
- **Deviations:** `scripts/lib` is not added to `cargo-inputs-hash.sh`: no cargo gate reads it, so hashing it would invalidate cargo-gate reuse for nothing. `check-gate-kit.sh` instead fails if a hashed cargo gate ever imports the kit while the hash does not name `scripts/lib`. The CLAUDE.md gate sentence is not extended: it no longer holds the per-gate list on this base (the list lives in `dev-docs/gates.md`, regenerated).
- **Does not subsume (follow-ups):** about 24 non-CSS gates that keep their own allowlist tables (demo-remote-assets' array, component-root-reset's typography-inherit list, the Rust and TS scanners); the theme "rule exists" regex assertions and the mutation text builders in several gates; `playwright/tools/css-rules.mjs`.
