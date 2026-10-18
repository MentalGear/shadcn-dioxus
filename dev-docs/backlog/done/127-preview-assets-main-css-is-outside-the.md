---
id: 127
title: "preview/assets/main.css is outside the stylelint gate's glob; 7 pre-existing errors"
status: "LANDED 2026-10-11 -- the real count was 68 errors across dx-components-theme.css (7)…"
size: "S"
area: build
updated: 2026-10-11
---

# Row 127: preview/assets/main.css is outside the stylelint gate's glob; 7 pre-existing errors

## Item

`preview/assets/main.css` is outside the stylelint gate's glob; 7 pre-existing errors

## Status / trigger

**LANDED 2026-10-11** -- the real count was 68 errors across `dx-components-theme.css` (7), `dx-effects.css` (54) and `main.css` (7), not 7. Fixed with no declaration change (a per-file postcss declaration diff matched, apart from the file-local `--_dx-*` -> `--dx-internal-*` rename the custom-property pattern requires); vendor-prefixed mask/clip declarations and the second `:root` ramp keep reasoned inline disables. `scripts/run-gates.sh` and CLAUDE.md now lint `"src/**/*.css" "assets/**/*.css"`; `.github/workflows/stylelint.yml` already linted `**/*.css`. Was: **OPEN** -- found 2026-10-03 when the chart round's `main.css` edits were linted by hand; the CLAUDE.md gate (`cd preview && npx stylelint "src/**/*.css"`) never sees it

## Size

S

## Detail

The gate's glob covers `preview/src/**/*.css` only; `preview/assets/main.css` (global page rules: scroll-behavior, touch-action, link colors, footer, the chart-adjacent rules from rows 117/118) is linted by nothing, and a manual run reports **7 errors** that predate this session. **Fix:** fix the 7 errors, then widen the gate to `"src/**/*.css" "assets/**/*.css"` in `CLAUDE.md` (and any script/workflow that runs it) so the class -- a stylesheet outside the glob -- cannot hide debt. Verify the widened glob by confirming it reports the seeded error before the fix, per the repo's "red first" rule.
