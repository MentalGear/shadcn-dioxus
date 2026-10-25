---
id: 429
title: "One gate kit: a shared CSS walker and one allowlist engine for the check scripts"
status: "BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P4)."
size: "M"
area: tooling
updated: 2026-10-25
---

# Row 429: One gate kit, a shared CSS walker and one allowlist engine

## Item

Eighteen `scripts/check-*.sh` heredocs each embed their own comment stripper, brace walker or selector splitter, and three of them already read the same input differently (quoted-string escapes, newline-preserving comment strip, rules nested in `@media`). About 40 scripts hand-implement "a reason is required, a stale entry fails, `--list` prints". Construction: `scripts/lib/css.py` (one quote-, paren- and escape-aware scanner returning `rules(css)`) and `scripts/lib/allow.py` (`Allowlist(entries)`, mandatory reason, `used` tracking, stale report). Two gates that read one stylesheet then read it one way by construction.

## Status / trigger

BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P4).

## Size

M

## Detail

Source: [final sweep note, P4](../research/final-sweep-2026-10-25.md). Scripts only, no product code. One lane owns `scripts/check-*.sh` while it runs.

- **Duplicated** (`grep -lE "^def (strip_comments|split_top|...)" scripts/check-*.sh`): `check-variant-scope.sh:108-141` (`style_rules`), `check-cross-component-overrides.sh:112-173` (`parse_rules`), `check-popup-boundary.sh:113-160`, `check-press-under.sh:46-62` (`rules`), `check-popup-root-reset.sh:107-117` (`find_rule`), `check-infinite-animations.sh:69-110` (`blocks`/`walk`), `check-hover-media.sh:73,188`, `check-component-root-reset.sh:150`, `check-demo-wrapper-width.sh:86,192`, `check-theme-preset-axes.sh:43`, `check-focus-outline-reset.sh:59`, `check-scroll-behavior.sh:84`, `check-theme-baseline.sh:143`, `check-cursor-baseline.sh:93`. `split_top` is defined in 10 scripts, `strip_comments` in 10; 18 scripts, 5,031 lines.
- **Drift (verified by reading):** `cross-component-overrides.sh:121-130` and `popup-boundary.sh:121-130` handle a backslash in quoted strings, `variant-scope.sh:115-123` does not (a `\"` in `content:` ends the quote early). `press-under.sh:46-47` strips comments without keeping newlines, so its reported lines are off. `popup-root-reset.sh:107-117` does not see a rule nested in `@media`/`@supports`. None is a known false negative today.
- **Subsumes:** the three drifts, roughly 1,000 duplicated lines, and the next gate's boilerplate. **Does not subsume:** the Rust-source scanners (`comment-scan.py` is already shared; `wrapper-props` and `attr-spread-collision-checker` are `syn` tools) and the shell-only gates.
- **Gate:** `check-gate-kit.sh` with fixtures for escapes, nested at-rules, comments containing newlines and `url(data:...;...{)`, plus a grep rule that no `check-*.sh` heredoc defines `strip_comments` or `split_top`. `check-gate-reuse.sh` and `cargo-inputs-hash.sh` add the lib to their declared inputs.
- **Risk:** a migration can loosen a gate silently. Each migrated gate keeps its mutation self-test as the oracle, and `run-gates.sh --list` output is captured before and after.
