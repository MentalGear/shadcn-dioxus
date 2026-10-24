<!-- Research report 2026-10-18 (read-only, Base UI 1.6.0 npm source, shadcn clone 30a58d4); BU/FM paths refer to the session-local unpack under /home/user/tgt/research-mk. -->

# Base UI 1.6.0 focus / interaction-type research (read-only; nothing in shadcn-dioxus edited)

BU = /home/user/tgt/research-mk/baseui/package (.mjs), BUU = research-mk/buu/package, FM = BU/floating-ui-react/components/FloatingFocusManager.mjs.
Nova = `git show HEAD:apps/v4/registry/styles/style-nova.css` (copy: /home/user/tgt/nova.css), shadcn HEAD 30a58d4.

## A. Interaction-type model

**Capture (per popup instance, not global).** `BU/utils/useOpenInteractionType.mjs:8-27` `useOpenMethodTriggerProps` wires the trigger's
`onPointerDown`+`onClick` through `BUU/useEnhancedClickHandler.mjs:11-37`. Pointerdown stores `event.pointerType` ("mouse"|"touch"|"pen");
a click with `event.detail === 0` is `'keyboard'` (:21-24); Safari/Firefox click lack pointerType so the last pointerdown type is reused (:30).
Only recorded when not already open (:11); iOS fallback `''`->`'touch'` for hit-slop taps (:12-16). Reset to `null` on close
(`useOpenInteractionType.mjs:37-41`, `utils/popups/popupStoreUtils.mjs:363-372`). Stored as `store.openMethod` (Dialog/Popover/Menu via
`DialogTrigger.mjs:58-60`, `PopoverTrigger.mjs:83`; Select/Menu/Combobox via the hook). **No `<html>` attribute; no global modality.**
Two other trackers exist: FM:197-214 (document capture `pointerdown`/`keydown` -> `lastInteractionTypeRef`, only to type the *close*) and
`useListNavigation.mjs:107,291,489,513` `isPointerModalityRef` (keydown=false, pointermove=true; gates scroll-into-view and hover/leave highlight).

**Consumers.**
- `FloatingFocusManager` gets `openInteractionType`; `initialFocus` may be a function `(openType)=>…` called at FM:363. Types in docs: `mouse|touch|pen|keyboard`.
- Default `initialFocus` (`popupStoreUtils.mjs:27-29`): `type==='touch' ? popup : true` ("true" = first tabbable else popup, FM:374-379). Doc reason:
  "opened by touch -- then the popup itself is focused to avoid opening the virtual keyboard". So mouse, pen and keyboard all focus the FIRST TABBABLE incl. text fields.
  - Dialog + AlertDialog: `DialogPopup.mjs:69` that default (AlertDialog reuses Dialog parts). Popover: `PopoverPopup.mjs:78` same.
  - Drawer: `DrawerPopup.mjs:278` always `popupRef` (no field focus on any modality).
  - Menu: `MenuPopup.mjs:118` `initialFocus: parent.type !== 'menu'` (root = true -> popup; submenu = false). No touch special-case; the item highlight is separate (below).
  - Select: `SelectPopup.mjs:385-390` passes NO initialFocus (FM default true); focus then goes to an item via useListNavigation.
  - Combobox/Autocomplete (shared `ComboboxPopup.mjs:96-100`): input in popup -> `touch ? popup : inputElement`; input as trigger -> `false` (typing target already focused).
    `ComboboxPopup.mjs:86-90` onFocus bounce-to-input is skipped for touch.
  - NavigationMenu: no FM; focus is moved by its root only.
- `finalFocus`/`returnFocus` (default true) FM:409-497: target = trigger (`domReference`), but `preferPreviousFocus = openInteractionType == null` (FM:417;
  null = programmatic open) prefers the element focused before open. **Returns focus on pointer and touch closes too**, via `.focus({preventScroll:true})` (FM:492).
  Suppressed: hover-leave close (FM:426), outsidePress unless `preventScroll` is supported (FM:429-446), focus-out close (FM:308 `preventReturnFocusRef`), and if focus already moved
  elsewhere (FM:487-491). NavigationMenu same set: `NavigationMenuRoot.mjs:19,131`. Menu: `MenuPopup.mjs:108-117` (root menu true; submenu only when trigger exists).
- Highlight on open (`useListNavigation.mjs`, default `focusItemOnOpen:'auto'`): sync effect :213 highlights an item on open only if `keyRef.current != null` (set at :529 by an
  Arrow/Enter/Space keydown on the trigger) or a virtual click (:498-506, screen readers). **Keyboard open -> first item (ArrowUp -> last, :230); pointer open -> NO highlight**,
  except Select, whose `selectedIndex` is highlighted on any open (:172-175, scrolled into view regardless of modality). Hover highlights (`focusItemOnHover`, `highlightItemOnHover=true`
  `MenuRoot.mjs:47`, `SelectRoot.mjs:67`); pointer leave clears it (:432-447) except touch. Combobox: `autoHighlight` false by default (`AriaCombobox.mjs:68`), `focusItemOnOpen:false` for selectionMode none (:854).
- Select item alignment: `alignItemWithTrigger=true` (`SelectPositioner.mjs:47`) but `mounted && … && openMethod !== 'touch'` (:76); touch also drops scroll arrows (`SelectScrollArrow.mjs:46`, `SelectList.mjs:47`) and gets
  scroll lock (`useAnchoredPopupScrollLock(…, openMethod==='touch')`, Popover/Menu/Combobox/Select positioners).
- `openOnHover`: Popover (default false) `PopoverTrigger.mjs:66` disabled when `openMethod==='touch' && reason===triggerPress`; Menu `MenuTrigger.mjs:135-137`.
- Typeahead: `useTypeahead` in Select (`SelectRoot.mjs:312-331`, works closed: changes value) and Menu (`MenuRoot.mjs:302`, resetMs); modality-independent.
- Misc: useClick `touchOpenDelay` for touch (`useClick.mjs:33`; Combobox 100ms, `AriaCombobox.mjs:824`); Safari blur of typeable element on unmount (FM:501-515).

## B. Focus ring in Base UI

- Unstyled. Docs (`BU/docs/react/overview/accessibility.md:32-35`): "it's the developer's responsibility to visually indicate focus ... typically ... `:focus` or `:focus-visible`".
- **No `data-focus-visible` anywhere** (grep: none). Exposed: `data-focused` (any focus, React state from onFocus/onBlur: Input `InputDataAttributes.mjs:29`, Field, Checkbox, Switch, Select trigger
  `SelectTrigger.mjs:135-156`, Slider, NumberField), and `data-highlighted` on Menu/Select/Combobox items (list active item). Tabs lost `data-highlighted`:
  "`:focus-visible` was already the recommendation in styles" (`docs/.../v1-0-0-beta-5.md:203`).
- Base UI's own demos: Input/Field `focus:outline-2 focus:-outline-offset-1` (plain `:focus`, `docs/react/components/input.md:31`); buttons `focus-visible:outline-2` (`quick-start.md:96`).
- Programmatic focus options: nearly all `preventScroll:true` (FM:389 only when focusing the popup itself; :492; `enqueueFocus.mjs:14`; `useListNavigation.mjs:129,259,421,452`).
  `focusVisible` (Chrome 144+ option) used in 3 places only: Slider thumb after key press `{focusVisible:true}` (`SliderThumb.mjs:323-326`), Slider pointer focus `{focusVisible:false}`
  (`SliderControl.mjs:248-251`), hidden-input autofill redirect `{focusVisible:true}` (`SelectRoot.mjs:420-423`), label `focusElementWithVisible` (`useLabel.mjs:66-71`). Never on popups.
- Pointer-open ring avoidance is **by reliance on the browser heuristic, not by code**: (1) touch -> popup (div tabIndex -1) focused; (2) pointer -> programmatic `.focus()` on a non-text element is
  not `:focus-visible` after a mouse press (Chromium/Firefox heuristic); (3) `useClick.mjs:92-95` comment: defer the open a frame "as an alternative to `event.preventDefault()` to avoid :focus-visible
  from appearing when using a pointer"; (4) `useLabel` `onPointerDown preventDefault`; (5) `matchesFocusVisible(el)` (`floating-ui-react/utils/element.mjs:48-58`) is READ to decide behaviour (useFocus hover-tooltips,
  Slider re-focus), Safari special case `useFocus.mjs:96-100`. Text fields still ring: Base UI does nothing about that.

## C. shadcn (Nova) on focus

- `base/ui/input.tsx:11` adds only `outline-none ...`; Nova `.cn-input` (nova.css:677): `border-input focus-visible:border-ring focus-visible:ring-ring/50 ... focus-visible:ring-3 transition-colors`.
  Same recipe: `.cn-textarea` :1337, `.cn-select-trigger` :1003, `.cn-native-select` :900, `.cn-button` :149 (all `focus-visible:`). `.cn-input-group` :1397 `has-[[data-slot=input-group-control]:focus-visible]`.
  `.cn-combobox-chips` :345 uses `focus-within:border-ring focus-within:ring-3` (ANY focus). Menu/select items: `focus:bg-accent` (:546,1032), combobox item `data-highlighted:bg-accent` (:309).
- **There is no separate non-ring "active" style for a field.** The only focus cues are border-colour -> `--ring` plus the 3px `ring/50` halo (and the caret). `transition-colors` only.
- Browser fact (Selectors-4 / HTML heuristic, "if the element supports keyboard input ... it matches"): a text-entry control (`input` text-like types, `textarea`, contenteditable) matches `:focus-visible` on
  ANY focus -- mouse click, touch tap, programmatic -- in Chromium, WebKit and Gecko. Hence shadcn fields show the full ring after a mouse click, and after Base UI's mouse-open focuses the first field.
  Buttons/links/triggers/div[tabindex] match only after keyboard (or script focus following keyboard/prior focus-visible).

## D. Us vs Base UI, recommendations

| Topic | Base UI | Ours |
|---|---|---|
| open modality capture | per-instance, trigger pointerdown/click (A) | global `data-dx-modality` on `<html>` (`modality.rs:55`), live, written by pointerdown/keydown/click capture |
| mouse open of dialog w/ field | first tabbable (field, ring) | panel itself (`modality.rs:73-100`, `lib.rs:573`); only `data-dx-autofocus` wins |
| touch | popup | panel, never editable (same) |
| pen | first tabbable | panel (grouped with touch) |
| return focus after pointer/touch | yes (`preventScroll`) | trigger refocus skipped for touch/pen (`lib.rs:416`); dialog opener refocus `lib.rs:626-631` |
| ring policy | UA `:focus-visible` heuristic | ring tokens zeroed unless modality=keyboard, `--dx-ring-field` exempt (`theme.css:1734-1738`, comment :1721-1723) |
| text field ring | full ring on any focus (heuristic) | `.dx-input:focus-visible` = muted fill + `--dx-ring-field` (`input/style.css:59-62`), so same as shadcn |
| select open highlight | keyboard: first/selected; pointer: selected only | keyboard: selected or first (`select/context.rs:77-85`); click open leaves focus on listbox, no highlight (matches) |

**Q2 answer.** We ring tapped/clicked text fields because shadcn does (C) and we copied it: `:focus-visible` is always true for text fields, and the theme explicitly exempts `--dx-ring-field` from the
modality zeroing (`theme.css:1721-1723`, "a text field the user tapped is still focused and still rings, as in shadcn"). The ring is not decoration on top of a separate "active" look: in shadcn it IS the
active look (border + halo), plus our extra `background-color: var(--dx-muted)` (`input/style.css:60`). The owner's intuition (active is shown already) is only true for the caret.

**(i) Mouse open -> first focusable incl. fields: confirmed as Base UI's default** (mouse, pen, keyboard -> first tabbable; only touch -> popup). We deliberately deviate (mouse -> panel). LIFT only together with (ii):
focusing a field on a mouse open then rings (shadcn parity) unless the quiet state exists. Where: `OPEN_FOCUS_JS` branch for `m==='mouse'` in `modality.rs:73-100`; pen follows touch today -> SKIP changing it.

**(ii) Text-field pointer focus: options.**
A. Keep full ring (status quo = shadcn). Pros: parity, zero risk, nothing changes visually vs upstream screenshots; clear target when a form has many fields. Cons: halo after every click feels loud (owner's complaint), wide halo can clip in tight groups.
B. Quiet pointer state: under `data-dx-modality=mouse|touch|pen` draw only the 1px `--dx-ring-color` edge + muted fill (drop the 3px halo); keyboard keeps `--dx-ring-field`. Where: add a `--dx-ring-field` override next to `theme.css:1734`
   (`--dx-ring-field: inset 0 0 0 1px var(--dx-ring-color)`), or better a new token `--dx-ring-field-pointer`. Pros: calm clicks; WCAG-OK (below); fits the existing "ring says keyboard" principle; uniform for Input, Textarea, Combobox,
   Select trigger, InputGroup, OTP (all consume the one token; `playwright/focus-ring.spec.ts` must then assert per modality).
   Cons: (1) **modality is live**: the first keystroke while typing sets `keyboard` (`modality.rs:55` keydown) so the halo would pop in mid-typing -- needs latching at focus time (e.g. a `focusin` capture writing
   `data-dx-focus-modality` on the target, or ignore character keys; this is the real cost); (2) touch-keyboard users with external keys fine; (3) a 1px edge alone must still meet 1.4.11 3:1 vs the field edge/background -- check
   `--dx-ring-color` against `--dx-input` in both themes; (4) diverges from shadcn so upstream diffs/screenshots differ (gate/spec churn).
   WCAG: 2.4.7 Focus Visible requires a visible indicator in "a mode of operation" where the keyboard is used -> B passes (keyboard keeps the full ring); the caret + border change still mark the pointer target.
   2.4.11 Focus Not Obscured (min.) concerns overlap by sticky/author content, unaffected by ring size (the keyboard-inset/scroll work covers it). 2.4.13 Focus Appearance (AAA, 2px/3:1 change) is the one that
   favours keeping the halo for keyboard. Never drop the border-colour change for pointer: colour-only-by-caret fails users who cannot see a 1px caret.
C. Border-colour only for pointer (no muted fill, no halo): quietest; same latching cost; weaker cue than B. Recommended if B is chosen: B (edge + fill), keyboard full ring, latched at focusin.
Recommendation: ask owner; if the loudness is the problem choose B with focus-time latching; otherwise A. Do NOT change keyboard behaviour.

**(iii) Other items.**
- LIFT: touch-only initial-focus exception already matches; add `pen` handling note only. LIFT the function-form `initialFocus(openType)`/`finalFocus(closeType)` idea as an optional prop on Dialog/Popover/Sheet
  (`dxOpenFocus` takes the modality; expose it) -- where: `modality.rs` + `dialog.rs`/`popover.rs`.
- LIFT: `preferPreviousFocus` for programmatic opens (FM:417): return focus to the element focused before open rather than a possibly missing trigger -- we already store `__dxOpener` (`lib.rs:570`); extend to non-dialog popovers.
- LEARN: Select highlights the selected item on a pointer open and scrolls it into view (BU `useListNavigation.mjs:172-175`); ours leaves the listbox focused with no highlight (`select/context.rs`, click path). Candidate for `select/components/trigger.rs`.
- LEARN: keyboard-vs-pointer detection via `event.detail===0` on click (`useEnhancedClickHandler.mjs:21`) is a cheap fallback to our `click` capture branch in `MODALITY_JS`.
- LEARN: Combobox/Autocomplete never auto-focus an input in the popup on touch (`ComboboxPopup.mjs:98`) -- same rule as `dxOpenFocus`; and `touchOpenDelay` 100ms to let the viewport settle (`AriaCombobox.mjs:824`) vs our `keyboard_inset.rs`.
- LEARN: `focusVisible:true|false` focus option (Chrome 144+; Safari/Firefox have it) is the platform way to say "this programmatic focus is/isn't keyboard-ish"; BU uses it only on slider thumbs and the autofill hidden input. Could replace some `focus-ok` sites
  in `lib.rs:416` but needs feature detection; keep the modality attribute as source of truth. SKIP for now.
- SKIP: Base UI's popup `tabindex=-1` + data-tabindex juggling (`handleTabIndex` FM:82-106), Safari typeable-blur on unmount, Drawer-always-popup (ours already panel-first for pointer).
- SKIP: `alignItemWithTrigger` (nothing equivalent in `preview/src/components/select`; popper-style select is fine, and BU disables it on touch anyway).
- SKIP: `openOnHover` default false parity -- we already have hover intent in `hover_intent.rs` / `hover_card.rs`.
