---
id: 304
title: "Radial chart tooltips share the gap, split equally per ring (lowest priority)"
status: "PARKED (lowest priority; ask the owner before starting), owner 2026-10-19. Ask the owner whether…"
size: "S–M (to scope)"
area: primitives
updated: 2026-10-19
---

# Row 304: Radial chart tooltips share the gap, split equally per ring (lowest priority)

## Item

On radial bar charts the tooltips should divide the gap between them, each ring's tooltip getting an equal share of it, instead of following the cursor.

## Status / trigger

PARKED (lowest priority; ask the owner before starting), owner 2026-10-19. **Ask the owner whether it's still wanted, and confirm the exact layout, before starting.**

## Size

S–M (to scope)

## Detail

- **Today:** radial uses `Follow::Pointer` (`chart.rs:583-587`), so the tooltip tracks the cursor (row 118's choice for polar charts).
- `radial::anchors` (`radial.rs:252-268`) computes per-ring anchors that a fixed placement could start from.
- The layout to confirm with the owner: the gap is divided equally, one slot per ring, and each ring's tooltip sits in its own slot.
