---
id: 149
title: "scroll_lock.rs installs non-passive wheel/touchmove listeners on window permanently after the first lock"
status: "LANDED 2026-10-05 (round 3; widened into and fixed by row 159) -- found 2026-10-04 by the…"
size: "S"
area: menus
updated: 2026-10-05
---

# Row 149: scroll_lock.rs installs non-passive wheel/touchmove listeners on window permanently after the first lock

## Item

`scroll_lock.rs` installs non-passive `wheel`/`touchmove` listeners on `window` permanently after the first lock

## Status / trigger

**LANDED 2026-10-05 (round 3; widened into and fixed by row 159)** -- found 2026-10-04 by the scroll-jank investigation (`dev-docs/archive/research/scroll-jank-2026-10-04.md` section 7.3); not fixed there because the file belonged to another lane

## Size

S

## Detail

`scroll_lock.rs::ensure_scroll_block_listeners_installed` adds both listeners at the first lock (the first Dialog, AlertDialog, Popover, DropdownMenu or ContextMenu open) and keeps them for the life of the page, gated afterwards by a `__dxScrollLocked` flag. After any overlay has opened once, every wheel and touch on the page again waits for the main thread before the compositor may scroll -- the cost this round removed from `#main` (Chromium `DidHandleInputEventSentToMain` 40/40 wheel ticks vs 0; wheel latency max 388 vs 143 ms with a busy main thread). **Fix:** add on lock, remove on unlock (refcounted if locks nest). `scripts/check-blocking-scroll-listeners.sh` allowlists `scroll_lock.rs` with a reason today; tighten that entry when this lands. The oracle (`oracle/tier2-html/scroll-main-thread.spec.ts`) inspects `#main` only because Playwright's own capture listeners pollute `window`: add an open-then-close-an-overlay step that reads the init shim's listener inventory, so "no permanent blocking listener" becomes testable. **Landed 2026-10-05 (round 3), through row 159's helper rather than a one-off fix:** `scroll_lock.rs`'s four listeners are installed by the outermost lock through the shared helper (`JsListeners`, passive unless the call says otherwise) and are torn down when the lock lets go, so no permanent blocking `wheel`/`touchmove` listener is left on `window` after an overlay has been opened. The lock now releases on the dialog's own `close` event: it released at unmount, ~250 ms after `close()`; the premise that it released at the `open` flip was wrong (row 141). **The oracle step this row asked for exists:** `playwright/oracle/tier2-html/listener-inventory.spec.ts` asks Chromium (CDP `DOMDebugger.getEventListeners` on `window`, `document`, `<html>`, `<body>`) for the listeners left after every modal, popover and menu consumer has been opened and closed twice, compared with a baseline taken BEFORE the first open, and asserts no non-passive `wheel`/`mousewheel`/`touchstart`/`touchmove` remains; it also pins the lock timing (a dialog whose exit is slowed to 1.2 s does not scroll and cancels the wheel until its real `close()`) and that the page width does not shift under a forced classic scrollbar. `scripts/check-blocking-scroll-listeners.sh` now also sees `listen(.., { passive: false })`; `scroll_lock.rs` keeps reasoned `blocking-ok` markers (the lock cancels the wheel by design).
