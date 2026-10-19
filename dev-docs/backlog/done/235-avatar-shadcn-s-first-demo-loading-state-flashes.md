---
id: 235
title: "Avatar: shadcn's first demo; loading state flashes"
status: "LANDED 2026-10-18 (lane demos 977a0624), merged on the session branch; deploy pending -- was…"
size: "S + M"
area: a11y
updated: 2026-10-18
---

# Row 235: Avatar: shadcn's first demo; loading state flashes

## Item

Avatar: shadcn's first demo; loading state flashes

## Status / trigger

**LANDED 2026-10-18 (lane demos 977a0624), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-18 in `dev-docs/research/next-round-plan-2026-10-15.md` (lane demos)** -- was: **BACKLOG (owner 2026-10-17)**

## Size

S + M

## Detail

Audit §6: shadcn's first demo = grayscale avatar, avatar with green badge, grayscale group "+3" (ours: 5 captioned avatars, `avatar/variants/main/mod.rs:20`). Group styling already matches (-8px overlap, 2px bg ring, grayscale). Error state correct (img removed, fallback shown). Loading NOT Base UI semantics: the fallback is removed when the status becomes `loading` (`primitives/src/avatar.rs:134,215`) and returns only on `loaded`/`error` -> initials, blank pulse, image. Fix: fallback shown while loading, Base UI's fallback `delay` prop; correct the false comment at `avatar/style.css:120`.
