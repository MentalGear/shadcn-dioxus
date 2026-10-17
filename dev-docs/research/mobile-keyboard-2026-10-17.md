<!-- Investigation report, 2026-10-17, read-only agent run against the 83758d5 debug SSG and the shadcn clone at 30a58d4; scratch paths under /home/user/tgt are session-local. -->

# Mobile keyboard / touch: what shadcn, Base UI, Radix, cmdk and vaul actually do

Read-only research. Sources: shadcn-ui/ui @30a58d4 (`git show HEAD:...`, worktree is sparse), npm tarballs unpacked under
`/home/user/tgt/research-mk/` (@base-ui/react 1.6.0, vaul 1.1.2, cmdk 1.1.1, @radix-ui/react-focus-scope 1.3.1,
react-remove-scroll 2.7.1, @floating-ui/dom 1.7.4). Treated as data, never executed. Our code was only read.
Not verified: any real iOS/Android behaviour (no device here); everything below is from source.

## 1. shadcn's own code: essentially nothing keyboard-specific
- Grep over `apps/v4/registry/bases/{base,radix,aria}/ui`, `new-york-v4`, `examples`, `content/docs`: **zero** hits for
  `visualViewport`, `inputMode`, `enterKeyHint`, `interactive-widget`, `virtualKeyboard`, `VirtualKeyboardProvider`, `initialFocus`
  (outside the vaul migration note), `onOpenAutoFocus` (outside docs).
- `registry/bases/base/ui/combobox.tsx:128` (radix flavour identical, also Base UI): popup is
  `max-h-(--available-height) w-(--anchor-width) max-w-(--available-width)`; Positioner gets only `side/sideOffset/align/alignOffset/anchor`
  (`:116`), so collision avoidance is Base UI's default (flip + shift, 5px padding). Same `max-h-(--available-height)` on Select (`select.tsx:94`),
  DropdownMenu (`:45`), ContextMenu (`:56`). Radix Popover/Select use Radix's `--radix-popper-available-height`.
- Input text size: `new-york-v4/ui/input.tsx:10` `text-base ... md:text-sm` (16px under md = no iOS focus-zoom). The v4 style sheets keep this (`style-vega.css:677 .cn-input`).
  Command input is `w-full text-sm` (`style-luma.css .cn-command-input`) -> 14px on phones, so iOS zooms on focus. We already floor both (below).
- `CommandDialog` (`bases/base/ui/command.tsx:36`): plain Dialog + `cn-command-dialog` (`rounded-xl p-0`, `top-1/3`), no autofocus prop on `CommandPrimitive.Input`;
  whatever focuses the input is Radix/Base UI dialog focus management (first tabbable). `.cn-command-list` = `max-h-72` fixed.
- Dialog (`dialog.tsx:~50`): `fixed top-1/2 left-1/2 -translate-x/y-1/2`, no max-height / keyboard handling. `dialog-scrollable-content.tsx:24`: `max-h-[50vh]`.
- `registry/bases/base/ui/drawer.tsx` is now **Base UI Drawer**, not vaul: overlay `min-h-dvh` (`:74`), `--drawer-content-max-height:calc(100dvh-6rem)` (`:130`);
  the wrapper does **not** use `Drawer.VirtualKeyboardProvider`. Legacy `new-york-v4/ui/drawer.tsx` is vaul with `max-h-[80vh]`.
- Only `useIsMobile` (`registry/bases/*/hooks/use-mobile.ts`, matchMedia `max-width:767px`) is used by the Sidebar (sheet on mobile). `apps/v4/hooks/use-media-query.tsx` powers the responsive examples.
- **Responsive pattern (c):** `registry/new-york-v4/examples/combobox-responsive.tsx` and `examples/{base,radix}/drawer-dialog.tsx` (docs: "Responsive Dialog",
  base `drawer.mdx:205`, radix `:117`): `const isDesktop = useMediaQuery("(min-width: 768px)")`; desktop = `Popover > Command(CommandInput, CommandList)`,
  else `Drawer > DrawerContent > <div className="mt-4 border-t"><Command .../>`. Only the legacy new-york-v4 Combobox has a Drawer variant; the base/radix
  combobox docs (`content/docs/components/{base,radix}/combobox.mdx`) say nothing about mobile or the keyboard. The Drawer there is vaul (legacy) so it gets
  vaul's `repositionInputs`; no `autoFocus`, so the Command input is NOT focused on open (vaul default) and the keyboard stays down until tap.
- `combobox-dropdown-menu.tsx:69` passes `autoFocus={true}` to CommandInput inside a DropdownMenu submenu (desktop-only idiom).
- Docs migration note (`base/drawer.mdx:285-297`): vaul `onOpenAutoFocus preventDefault` -> `initialFocus={false}`; `repositionInputs` "has no one-to-one replacement".
  Takeaway: shadcn leaves mobile keyboard entirely to the primitive libraries. The useful code is in Base UI and vaul.

## 2. Base UI 1.6.0 (the real source of mobile behaviour)
**Interaction-type-aware initial focus (finding a): verified, and it is the DEFAULT.**
- `utils/popups/popupStoreUtils.js:46`: `createDefaultInitialFocus(popupRef) = (interactionType) => interactionType === 'touch' ? popupRef.current : true`.
  Comment: "focuses the popup element itself to prevent the virtual keyboard from opening (required for Android specifically; iOS handles this automatically)".
  Used by `popover/popup/PopoverPopup.js:84`, `dialog/popup/DialogPopup.js:75` (and AlertDialog). Docs state it for Popover/Dialog `initialFocus`.
- Combobox with the input *inside* the popup: `combobox/popup/ComboboxPopup.js:105`
  `interactionType => interactionType === 'touch' ? popupRef.current : inputElement`; and `onFocus` (`:94`) only bounces focus to the input when `openMethod !== 'touch'`.
  Input-as-anchor combobox / Select / Menu: `initialFocus` is not touch-aware (the input is the trigger the user just tapped). Drawer/Combobox docs: no touch default documented (Drawer default = first tabbable).
- Type: `'mouse' | 'touch' | 'pen' | 'keyboard' | ''`. Source: `utils/useOpenInteractionType.js` (trigger `pointerdown` pointerType via `useEnhancedClickHandler`;
  on iOS Safari an unclassified tap (hit-slop, no pointerdown) is forced to `'touch'`, `:23`). Consumed in `FloatingFocusManager.js:368`
  (`initialFocus(openInteractionType || '')`, then `enqueueFocus(el, {preventScroll: el === floatingFocusElement})`). `false` = no focus move.
- Touch-opened anchored popups skip page scroll lock unless the popup is ~full width (`utils/useAnchoredPopupScrollLock.js`, 20px tolerance) so users can swipe outside to dismiss. Select disables align-item-with-trigger on touch (`SelectPositioner.js:82`).

**Positioning with the keyboard up (finding b), all via Floating UI:**
- `utils/useAnchorPositioning.js`: middleware order `offset, flip, shift, size, arrow, hide`; defaults `collisionPadding 5`, `collisionAvoidance side:'flip' align:'flip'`;
  `size().apply` writes `--available-width/--available-height/--anchor-width/--anchor-height` (`:217-230`); comment: "Ensure the popup flips if it's been limited by
  its --available-height ... flip() padding is larger than size() padding". `lazyFlip` (`:381`): lock the flip side so a filtering list doesn't ping-pong.
- No own `visualViewport` listener in the positioner. It comes free from Floating UI: `@floating-ui/dom getViewportRect` uses `visualViewport.width/height` and (WebKit, `strategy: fixed`) its `offsetLeft/Top`
  (`floating-ui.dom.mjs:223-237`); `getOverflowAncestors` adds `win.visualViewport` to the autoUpdate scroll/resize targets (`floating-ui.utils.dom.mjs:156`). So keyboard open => resize event => re-run
  flip/shift/size against the visible region => popup flips or shrinks (capped by `--available-height`). Our `reposition()` already does this and goes further (exclusion zone, no main-axis shift).
- `utils/adaptiveOriginMiddleware.js:46`: for `strategy fixed` uses `visualViewport.width/height` to compute transform-origin sides. `select/popup/SelectPopup.js:307`: if `visualViewport.scale !== 1 && webkit` -> disable item-aligned placement (Safari mis-positions when pinch-zoomed).
  `@base-ui/utils useScrollLock`: `overflow:hidden` locking, skipped when WebKit is pinch-zoomed (`:80`). No `position:fixed` body trick.

**Drawer.VirtualKeyboardProvider (new, 1.6.0): the best current prior art.** `drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.js`, docs `docs/react/components/drawer.md:1697` ("Virtual keyboard aware").
- Wrap a bottom sheet; needs `Drawer.Viewport`. While open it listens to `visualViewport` `resize`+`scroll` and document `focusin/focusout` (capture).
- "Keyboard" = `innerHeight - visualViewport.height > 60` (`KEYBOARD_RESIZE_THRESHOLD`) and `visualViewport.scale === 1` (suspended while pinch-zoomed); smaller changes = browser chrome.
- Publishes `--drawer-keyboard-inset` (= `innerHeight - vv.bottom`) on the viewport element; docs: always `var(--drawer-keyboard-inset, 0px)` with fallback. A pinned footer input is lifted by it.
- For the focused field: finds the nearest vertical scroll container, adds `padding-bottom += overlap + 48px` (`KEYBOARD_SCROLL_SLACK`) with `overflow-anchor:none`, then smooth-scrolls (`auto` if reduced motion)
  so the field centres in the visible band (16px margin). All restored on blur/close.
- Touch path: on `touchend` over a text input (hit-slop probe +-16px, taps cancelled if moved >10px or landing on another interactive element) it `preventDefault`s, focuses the input *synchronously in the gesture*
  (iOS only opens the keyboard then) with `opacity:0; transform:translateY(-2000px); transition:none` for that one call + `focus({preventScroll:true})`, then re-dispatches a synthetic `click`.

## 3. Radix
- `react-focus-scope` (`dist/index.mjs`): on mount, if focus is not already inside, dispatches `focusScope.autoFocusOnMount` (cancelable = `onOpenAutoFocus`), else `focusFirst(tabbables, {select:true})`, else the container.
  **No pointer-type awareness**: a tap opens a Dialog with the first input focused => keyboard pops. `focus()` uses `{preventScroll:true}` (`:297`). Dialog/Popover forward it as `onOpenAutoFocus` (popover `:263`).
- No `visualViewport` in react-dialog / react-popover. Popper = Floating UI `size()` -> `--radix-popper-available-height` (same viewport-rect behaviour as above).
- `react-remove-scroll` (`dist/es2015/SideEffect.js:133`): non-passive document `touchmove`/`wheel`/`touchstart` listeners that `preventDefault` when the gesture target is outside the lock (scrollable-ancestor aware, pinch-zoom allowed via `allowPinchZoom`); plus `RemoveScrollBar` (overflow hidden + gap padding). No iOS-specific position:fixed and no keyboard code. Contrast vaul (below).

## 4. cmdk 1.1.1
`Command.Input` (`dist/index.mjs`): renders `autoComplete="off" autoCorrect="off" spellCheck={false} role=combobox aria-autocomplete=list type=text`. **No autoFocus, no `autoCapitalize`, no `inputMode`/`enterKeyHint`, no touch code.**
Item `scrollIntoView({block:'nearest'})` only. Input focus in a CommandDialog is purely the dialog's focus-first behaviour.

## 5. vaul 1.1.2 (iOS keyboard mechanism, precisely)
All in `dist/index.mjs` (prettified copy `/home/user/tgt/research-mk/vaul/vaul.pretty.js`).
1. **No autofocus on open**: `autoFocus=false` default; `onOpenAutoFocus` is `preventDefault()`ed unless `autoFocus` (`:1902`). So a vaul sheet opens without raising the keyboard.
2. **`repositionInputs` (default true)** resize handler on `window.visualViewport` `resize` only (`:1422-1496`), active when `isInput(document.activeElement) || keyboardIsOpen`:
   - `keyboardHeight = innerHeight - visualViewport.height`; `keyboardIsOpen` toggles when it changes by >60px between calls.
   - If drawer taller than the visual viewport (or keyboard open): `height = min(height, vvHeight - (tallEnough(>80% of innerHeight) ? offsetTop : 26 /*WINDOW_TOP_OFFSET*/))`, floored at `vvHeight - offsetTop`; `fixed` mode only shrinks height.
   - `drawer.style.bottom = max(keyboardHeight,0)px` (moves the bottom-anchored sheet above the keyboard; with snap points only when keyboard open). Keyboard closed: restore `initialDrawerHeight` (skipped on mobile Firefox).
   - Pure inline-style writes on the drawer element, no CSS variable.
3. **`usePreventScroll` / `preventScrollMobileSafari`** (react-spectrum derived, iOS only, `:149-300`): document capture touchstart/move/end (non-passive) to block window scroll and over-scroll at a nested scroller's ends;
   `touchend` on a text input -> `preventDefault`, `translateY(-2000px)` on the input, focus it synchronously, remove the transform next frame (so Safari does not scroll the page to centre it), then `scrollIntoView` it manually after the next visualViewport resize;
   body offset by scroll position with negative margin and `window.scrollTo(0,0)`; `window scroll` => scrollTo(0,0). (Base UI's provider is the cleaner descendant of step 3's input trick.)
4. `usePositionFixed` (`:955`): on Safari `body{position:fixed;top:-scrollY}` while open, restore + `scrollTo` on close, bottom-bar correction after 300ms.
5. **`--initial-transform`** (`:65` CSS) is only the closed-position offset for the snap-point slide animation, `translate3d(0, var(--initial-transform,100%), 0)`; it is NOT part of the keyboard logic. `--snap-point-height` likewise.

## 6. Map to our code
Already in place (so SKIP re-porting): `top_layer.rs` `reposition()` uses `viewportMetrics()` (visualViewport size + offset, scrollbar-excluded), listens to `visualViewport` resize/scroll (`:1955`),
exclusion-zone side choice + `--dx-available-height/-width` (`:1236-1260`, `combobox/style.css max-height:min(18rem,var(--dx-available-height,18rem))`), a ResizeObserver for typing growth.
`use_dialog_open_driver` (`lib.rs:507`) already implements Base UI's touch rule for every native-`<dialog>` modal (row 206): opener matches `:focus-visible` => first control, else panel focused with children `inert` during `showModal()`; `[autofocus]` wins.
Inputs are already floored to 16px on touch (`input/style.css:80`, `command/style.css:245`), `spellcheck=false`, `autocomplete=off` (`combobox/components/input.rs:116-130`, `command.rs:494`).

| # | Finding | Verdict | Where / how |
|---|---|---|---|
| a1 | Touch -> focus the popup, not the input (Base UI default) | **SKIP for modals (done, row 206)**; **LIFT for the non-modal Popover arm** and input-inside-popup Combobox/Select-with-search if any | `popover.rs` non-modal arm (focus moves into content on open): reuse the same `:focus-visible`-on-opener read at open time; popup gets `tabindex=-1`. Check `[data-dx-...]` search inputs inside popovers (Date Picker, Combobox-in-popover). |
| a2 | CommandDialog autofocuses input on every open (shadcn/cmdk/our `autofocus` default) | **LEARN / owner call** | Row 206 deliberately keeps it. Base UI/vaul default is opposite on touch. Option: `CommandInput` autofocus only when opener matched `:focus-visible` or a hardware-keyboard shortcut (`default_open` + no coarse pointer: `matchMedia('(pointer:coarse)')`). Keep `[autofocus]` for desktop. |
| b1 | Keyboard inset variable + scroll-focused-field-into-view for dialog-engine modals (Base UI `VirtualKeyboardProvider`) | **LIFT** (highest value gap) | New helper next to `VIEWPORT_METRICS_JS` in `top_layer.rs` (reuse its visualViewport math), called from `use_dialog_open_driver` after `showModal()` and torn down on close (use `js_listener.rs`'s `listen`, per the js-listeners gate). Set `--dx-keyboard-inset` = `innerHeight - (vv.offsetTop+vv.height)` when diff > 60px and `vv.scale===1`; on `focusin` of text inputs find nearest scroller, add `padding-bottom += overlap+48`, smooth-scroll to centre (respect reduced motion), restore on blur/close. Styles consume it: `drawer/style.css` bottom `max-height` -> `min(80%, calc(100% - var(--dx-keyboard-inset,0px)))`, sticky footer `padding-bottom + var(--dx-keyboard-inset,0px)`. Today `drawer/style.css:142` `max-height:80%` and dialog `top:50%` ignore the keyboard entirely (same gap as shadcn). Needs a motion/visibility gate review (`check-motion-gating`) only if animated. |
| b2 | `size()`-style available-height capping for anchored panels | **SKIP (done, row 210)** | `--dx-available-height` already published; confirm every panel sheet with a fixed cap (Select, DropdownMenu, Popover with lists) reads it, as shadcn does with `max-h-(--available-height)` on all menus. Quick audit candidate: grep sheets for fixed `max-height` without the var. |
| b3 | `lazyFlip` (lock flip side while the list filters) | **LEARN** | Our row 210 keeps the preferred side while it fits; confirm typing doesn't oscillate side when room is marginal (spec already sweeps typing grow/shrink). |
| b4 | Suspend keyboard logic while pinch-zoomed (`vv.scale !== 1`), 60px threshold to ignore browser-chrome resizes | **LIFT (small)** | Apply to any new keyboard helper; check `viewportMetrics()`/`reposition()` already tolerate zoom (row 210 has a CDP pinch case). |
| b5 | vaul `bottom = keyboardHeight` + shrink-to-visual-viewport height on the sheet | **SKIP** | Imperative style writes and a toggling `keyboardIsOpen` heuristic; Base UI's CSS-variable + scroll-padding design is cleaner and composes with our token system. Keep as fallback reference only. |
| b6 | iOS tap-focus trick (translateY(-2000px) + synchronous focus in `touchend` + synthetic click; vaul/Base UI) | **LEARN, do not lift yet** | Fixes Safari scrolling the page when focusing an input inside a transformed/`fixed` sheet. High complexity, needs a real iPhone to validate (none here). Only if the manual device step for b1 shows page jump. |
| b7 | vaul `usePositionFixed` / `preventScrollMobileSafari` | **SKIP** | Our `scroll_lock.rs` has a long history (generations 1-3, gutter/scrollbar); native `<dialog>` modal already inerts the page. Revisit only on a reproduced iOS scroll-through. |
| c1 | Responsive Popover(Command) on desktop / Drawer(Command) under 768px (`combobox-responsive.tsx`, `drawer-dialog.tsx`) | **LEARN; optional LIFT as a demo, not a primitive** | shadcn only offers it as a legacy example (Drawer = vaul). Their base/radix docs do not recommend it for Combobox. We can ship an example using `use_media_query`-equivalent + our Drawer + `Command` (inline, no autofocus: our inline `CommandInput` already does not autofocus). It sidesteps anchored placement entirely on phones. Benefit grows once b1 lands (Drawer with a keyboard-aware body). Decision for the owner: demo vs. fix-in-place. |
| d1 | `autocapitalize="off"` (+ `autocorrect="off"`) on filter inputs; cmdk sets autoCorrect off but no autocapitalize | **LIFT (tiny)** | Add `autocapitalize: "off"` beside `spellcheck` in `combobox/components/input.rs:116` and `command.rs:494` so iOS does not capitalise the first letter of a filter. `enterkeyhint="search"`/`"done"` (nobody in shadcn sets it): optional polish, Combobox `done`, Command `go`/`search`. `inputmode` stays default (text). |
| d2 | `interactive-widget=resizes-content` viewport meta / `navigator.virtualKeyboard` | **LEARN (from memory, unverified here)** | Chromium-family opt-ins that make the layout viewport shrink for the keyboard, which would make `100%`/`dvh` sheets work with no JS; Safari did not support them when last checked. Not used by shadcn or Base UI. Not a replacement for b1 on iOS. |
| d3 | `dvh` units for full-height overlays | **LEARN** | shadcn Drawer uses `min-h-dvh`/`100dvh-6rem`. `dvh` follows browser chrome (URL bar), NOT the on-screen keyboard on iOS; do not expect it to solve b1. Our Sheet/Drawer use `%` of the fixed containing block; fine. |

## Suggested order
1. d1 (5 lines, no risk). 2. b1 as one lane: helper in `top_layer.rs` + `use_dialog_open_driver` hook + `drawer/style.css`/`dialog/style.css` consumers + a Playwright spec that emulates a keyboard by
   shrinking `visualViewport` (we have the CDP pinch-zoom harness from `combobox-placement.spec`; note it is Chromium-only, so a manual iPhone step is mandatory as in row 210).
3. a1 audit of the non-modal Popover arm. 4. a2/c1 are owner decisions (behaviour change vs demo).
Key files: `primitives/src/top_layer.rs`, `primitives/src/lib.rs:507`, `primitives/src/popover.rs`, `primitives/src/command.rs:494`, `primitives/src/combobox/components/input.rs:116`, `preview/src/components/drawer/style.css:142`.
