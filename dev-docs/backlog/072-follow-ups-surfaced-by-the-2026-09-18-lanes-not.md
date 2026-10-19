---
id: 72
title: "Follow-ups surfaced by the 2026-09-18 lanes, not built this round"
status: "Surfaced 2026-09-18 across lanes C, E, B and integration round 1"
size: "—"
area: process
updated: 2026-09-18
---

# Row 72: Follow-ups surfaced by the 2026-09-18 lanes, not built this round

## Item

Follow-ups surfaced by the 2026-09-18 lanes, not built this round

## Status / trigger

Surfaced 2026-09-18 across lanes C, E, B and integration round 1

## Size

—

## Detail

**Navigation Menu's oracle coverage** — `top-layer.spec.ts`'s `CONFORMING_CASES` list may want its own `navigation_menu` entry now that the primitive is anchored on the same engine (Lane C's own note); trigger: whenever `top-layer.spec.ts` next gets a new consumer pass. **[Done 2026-09-18/19, batch 2, row 81 — `8f3b10e` gave NavigationMenu the identical 7-location footprint Navbar already carries.]** **Select's `scroll_lock=false` fixture** — row 9 shipped the opt-out prop with no Playwright case exercising it (Lane E's own documented gap); trigger: next time `scroll-lock.spec.ts` is touched. **[Done 2026-09-18/19, batch 2, row 81 — `8f3b10e` added a raw-`Select` `scroll_lock: false` fixture case to `scroll-lock.spec.ts`.]** **Navigation Menu's deliberately-out-of-scope keyboard surface** — Home/End and in-panel Up/Down roving inside an open panel, and a `NavigationMenuIndicator`, both deliberately not implemented this round (Lane C); trigger: a future APG-completeness pass on this primitive, or a user report. **[Done 2026-09-18/19, batch 2, row 77 — `08a14d8` added Home/End, panel Up/Down stopping at the ends, and ArrowDown-opens-and-focuses-first; a `NavigationMenuIndicator` was not added, still open.]** **Drawer's native/Blitz arm** — Lane B's own account covers only the web arm's drag mechanics (`translate`, pointer tracking via `pointer.rs`); consistent with this repo's standing native/Blitz gap (row 23), drag tracking here likely depends on `document::eval` the same way the rest of the crate does, and was not exercised on that renderer; trigger: row 23 (whenever native/Blitz becomes a target). **`dx serve` hot-reload's third trap** — a structural Rust logic change can log only `Hotreloading: <file>` with no `Build completed` line and silently not take effect, and only a cold start logs "Build completed successfully" (a watch rebuild logs "Build completed in Ns") — full account now in `dev-docs/dx-serve-hot-reload.md`; trigger: already written up, no further action needed here. **Playwright positional args are substring matches** — `npx playwright test select.spec.ts` also runs `native_select.spec.ts` (round 1's own target-spec run swept this in); trigger: whenever composing a targeted Playwright invocation, check the actual file list it resolves to before trusting a "14 target specs" count. **`pkill -x dx` blast radius** — kills every `dx` process in the container, not just the intended one (it killed Lane J's own `:8082` server once, mid-session, unexplained until traced); prefer killing by PID or `fuser -k <port>/tcp`; trigger: any time multiple `dx serve` instances coexist (parallel lanes, a leftover background server). **Finish the `PLAYWRIGHT_BASE_URL` migration** — Lane K's `playwright/base-url.ts` (round 3) migrated 4 of the suite's specs (avatar, toolbar, sidebar, drag_and_drop_list) as a proof of concept, consolidating three pre-existing ad hoc `BASE_URL`/`BASE` patterns into one, default behaviour unchanged; the remaining ~65 specs still carry their own ad hoc pattern or none; trigger: whenever a spec file is next touched, or a dedicated migration pass. **[Done 2026-09-18/19, batch 2, row 81 — `e0bb23f` + `2cb1b06` + `3cc0881` migrated every remaining spec; every executable occurrence of the old hardcoded literal is gone.]** **Move component stylesheets so CSS edits hot-reload** — Lane K measured a themed component's `style.css` under `src/` costing a full ~28s rebuild versus ~0.9s for `preview/assets/` CSS, which does hot-reload; trigger: a dev-loop-focused pass, or when component-CSS iteration speed becomes a bottleneck (full account: `dev-docs/dev-loop.md`, row 76). **[Superseded 2026-09-18/19, batch 2, row 80 — `238c8f4` found the real root cause was the compile-time `code!()` embed, not file location; moving files would not have fixed it.]** **Lanes should use `npm ci`, not `npm install`** — Lane C (round 3) found a fresh `npm install` in its worktree resolved a Playwright expecting Chromium 1223, while `/opt/pw-browsers` only has 1194 (the main checkout's `npm ci` from the lockfile matches 1194); trigger: every future worktree/lane setup step. **[Done 2026-09-18/19, batch 2, row 79 — the guards lane (`f11b98b`) added this rule to `CLAUDE.md`'s gate section.]** None of these block anything landed this session; listed so a fresh session doesn't have to re-discover them from the lane hand-backs
