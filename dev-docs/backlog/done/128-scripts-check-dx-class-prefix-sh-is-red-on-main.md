---
id: 128
title: "scripts/check-dx-class-prefix.sh is red on main @ 62de3cb: bar_chart/style.css and line_chart/style.css…"
status: "LANDED 2026-10-03 (follow-up to PR #81, see the fix note at the end of this row) -- found…"
size: "S"
area: theme
updated: 2026-10-03
---

# Row 128: scripts/check-dx-class-prefix.sh is red on main @ 62de3cb: bar_chart/style.css and line_chart/style.css…

## Item

`scripts/check-dx-class-prefix.sh` is **red on `main` @ `62de3cb`**: `bar_chart/style.css` and `line_chart/style.css` select `.dx-card`, `.dx-card-header`, `.dx-card-content` (lines 1, 4, 65, 71, 89 in each)

## Status / trigger

**LANDED 2026-10-03** (follow-up to PR #81, see the fix note at the end of this row) -- found 2026-10-03 by running every `check-*.sh` after PR #81 merged (rc=1, 10 errors + 2 `top_layer` warnings); introduced by `0532e5a` (the shadcn-registry demo port: interactive-header rules that override the Card's own classes). PR #81's deploy commit reported "13 check scripts green"; this one is not

## Size

S

## Detail

The gate requires every themed component stylesheet to define only `dx-<component>-...` classes (row 32: the collision safety `#[css_module]` hashing used to provide). The two stylesheets deliberately use two-class selectors on the *card's* classes to beat the Card's own specificity (their header comments say so), which is precisely what the gate forbids. **Fix by construction, not by exempting the files:** give the interactive card its own class (`.dx-bar-chart-card`, `.dx-line-chart-card`, applied to the `Card` in the interactive demo) and select `.dx-bar-chart-card .dx-card-header` etc. -- a *descendant* of a namespaced class -- or check how the gate treats the descendant form first; re-run the chart and gallery specs (`charts_gallery.spec.ts`, the `*chart*.spec.ts` set) and compare the interactive header's computed styles before and after, because specificity is the whole point of those rules. The `top_layer` warnings (`.dx-anchor-*-native`) are pre-existing and only warn. This is the failure `CLAUDE.md`'s "run every gate before every commit" rule exists for (row 67: a merged PR left a gate red while CI is frozen, row 33). **Fix as landed (2026-10-03):** the descendant form suggested above would NOT pass -- the gate flags any `.class` in selector position, including inside a compound or descendant selector. The construction that passes is the one `chart/style.css` already used for the footer (`[data-slot="card-footer"].dx-chart-footer`): one shared rule set in `chart/style.css` selecting `[data-slot="card"\|"card-header"\|"card-content"].dx-chart-interactive[-header\|-content]` -- attribute selectors on the Card's own `data-slot` plus an own class, same (0,2,0) specificity -- used by both the bar and line interactive demos; `bar_chart/style.css`/`line_chart/style.css` are now rule-less stubs (the `examples!` macro needs the files). Evidence: all 13 `check-*.sh` green; fmt/clippy/`cargo test --workspace` (670+73+170) green; SSG release build, 88/88 routes; interactive headers on `/charts/{bar,line}/` and `/component/{bar,line}_chart/` at 1440/390 light/dark: 0 differences across 128 element boxes/computed styles vs the previous build; `charts_gallery` + `bar_chart` + `line_chart` specs 75/75.
