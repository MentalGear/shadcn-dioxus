---
id: 231
title: "Docs Button page looks different from shadcn's base docs"
status: "LANDED 2026-10-18 (lane docs-site 37db3980), merged on the session branch; deploy pending -- was…"
size: "S"
area: docs
updated: 2026-10-18
---

# Row 231: Docs Button page looks different from shadcn's base docs

## Item

Docs Button page looks different from shadcn's base docs

## Status / trigger

**LANDED 2026-10-18 (lane docs-site 37db3980), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-18 in `dev-docs/research/next-round-plan-2026-10-15.md` (lane docs-site)** -- was: **BACKLOG (owner 2026-10-17)**

## Size

S

## Detail

Audit §2: shadcn docs use `base-nova`; size, radius, padding, font match. Differences: (a) text tokens -- the site never sets `--foreground`, `--primary-foreground`, `--secondary-foreground`, `--destructive`, so the theme's old fallbacks win (`dx-components-theme.css:192,205,207,229`): primary text #fff vs #fafafa, secondary #000 vs #171717, outline/ghost #111 vs #000, dark text #d4d4d4 vs #fafafa; (b) destructive solid vs shadcn's 10% tint -- deliberate (row 134), owner to confirm; (c) most visible: the demo FRAME (dotted grid, card bg #171717 dark, 8px radius, 32px padding) vs shadcn's plain `rounded-2xl`, 288px tall, 40px padding (`main.css:1345`); (d) first demo: shadcn one outline button + icon button, ours 6 variants. Fix: set the shadcn tokens on the site, restyle the frame, shadcn's first demo.
