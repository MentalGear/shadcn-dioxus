# Chat kit demos: shadcn originals vs ours (read-only research, 2026-10-14)

> Provenance: produced by a read-only research agent on 2026-10-14 from shadcn-ui/ui main @30a58d4. Copied here from the agent's working notes; summarised in backlog row 213.

Scope: `preview/src/components/{message,bubble,marker,attachment,message_scroller}/` (docs.md, component.json, component.rs, every `variants/*/mod.rs`) against shadcn/ui's chat docs and examples. No repo files edited; no cargo/dx run. Scratch copies are in the session scratchpad (`shadcn-mdx/`, `shadcn-ex/`), not in the repo.

## 0. Sources, and what worked

| Source | Result |
|---|---|
| `https://ui.shadcn.com/docs/components/{message,bubble,marker,attachment,message-scroller}` (and `/components/base/...`) | 200 (~0.9-1.9 MB Next.js HTML). Both paths serve identical bytes. The HTML embeds only the default demo name, so it was used for confirmation, not for the example list. |
| `.../components/chat`, `.../base/chat` | 404 (no chat page). |
| `raw.githubusercontent.com/shadcn-ui/ui/main/apps/v4/content/docs/components/base/<name>.mdx` | **200, the one that worked.** Each `<ComponentPreview name="...">` list was taken from these five files. Non-`base/` paths are 404. One early request returned 000 (transient); a retry worked. |
| `raw.githubusercontent.com/.../main/apps/v4/examples/base/<name>.tsx` | 200 for all 44 example files (40 referenced from the docs, 4 examples-only). |
| Support files `apps/v4/lib/message-animations.ts`, `apps/v4/components/message-animated.tsx`, `apps/v4/components/markdown.tsx`, `apps/v4/lib/ai.ts` | 200. |
| Local clone `/home/user/shadcn-ui/ui` (HEAD `30a58d4`, 2026-10-12, sparse) | Used as a cross-check only: 7 sampled example files are byte-identical to `main`. |

Not verified: the registry sources (`apps/v4/registry/bases/base/ui/{message,bubble,marker,attachment,message-scroller}.tsx`) were not diffed against our components line by line. Shadcn's own line counts (catalog section 3) are at snapshot `295a1f1`.

Provenance inside this repo (what our components cite):
- `preview/src/components/message_scroller/style.css` line 1 cites `shadcn-ui/ui@295a1f1 registry/bases/base/ui/message-scroller.tsx` and `registry/styles/style-nova.css`.
- `primitives/src/message_scroller.rs` (module doc) cites `dev-docs/research/shadcn-catalog-2026-10-04.md` section 4 and `@shadcn/react` (MIT). `primitives/src/message_scroller.js` cites `use-message-scroller-controller.ts`, `use-message-scroller-commands.ts`, `geometry.ts`.
- The other four `component.json` files cite only `https://github.com/MentalGear/shadcn-dioxus` (dependency git URLs). Their `docs.md` files say "styled composition", with no shadcn URL.
- `dev-docs/research/shadcn-catalog-2026-10-04.md` sections 3.1-3.5 describe all five components. `dev-docs/archive/rounds/component-backlog.md` rows 81-84 record Message, Bubble, Marker and Attachment as "built 2026-10-04 (chat kit)".
- **Correction to the record:** `dev-docs/research/shadcn-demo-parity-audit-2026-10-11.md` (line 45 and line 57) says "No original page: ... message, bubble, marker, attachment, message_scroller". That is wrong. All five have docs pages and examples in `main` today (and in the local clone at `30a58d4`). The owner's "far richer" remark is consistent with this.

## 1. Inventory

| Component | shadcn docs previews | shadcn examples-only | Ours (demos) |
|---|---|---|---|
| message | 6 | 1 (`message-markdown`) | 6: main, avatar, group, header_footer, actions, attachment |
| bubble | 9 | 1 (`bubble-markdown`) | 5: main, alignment, group, reactions, link_button |
| marker | 8 | 1 (`shimmer-marker`, same content as `marker-shimmer`) | 4: main, variants, status, link_button |
| attachment | 6 | 0 | 6: main, states, sizes, image, group, trigger |
| message_scroller | 11 | 1 (`message-scroller-state`) | 4: main, last_anchor, load_history, long |
| **Total** | **40** | **4** | **25** |

Our `long` variant (2,000 rows through `MessageScrollerRows`) has no shadcn counterpart, and neither does the `MessageScrollerRows` API. Shadcn's own virtualization example uses TanStack Virtual and is not a demo.

## 2. The richest shadcn demos

**`message-scroller-demo`** (the docs hero) is a full chat: `Card` with a header, a `Reset` button wrapped in a `Tooltip`, an `Empty` state, a read-only composer (`InputGroup` with a `DropdownMenu` "+" menu of attachments and a send `InputGroupButton`), a simulated `useChat` stream, and `MessageAnimated` rows. Core JSX:

```tsx
<MessageScrollerProvider>
  <MessageScroller>
    <MessageScrollerViewport>
      <MessageScrollerContent aria-busy={isBusy} className="p-(--card-spacing)">
        {messages.map((message) => (
          <MessageAnimated key={message.id} message={message}
            scrollAnchor={message.role === "user"} />
        ))}
      </MessageScrollerContent>
    </MessageScrollerViewport>
    <MessageScrollerButton />
  </MessageScroller>
</MessageScrollerProvider>
```

**Option and toggle demos** (the ones that let the reader change settings):

- `message-scroller-anchoring`: a `ToggleGroup` ("User" / "Assistant") sets which role gets `scrollAnchor`, then resets the thread. Calls `scrollAnchor={message.role === anchorRole}`.
- `message-scroller-opening-position`: `Tabs` over `start` / `end` / `last-anchor`. Changing it bumps a `key`, which remounts the scroller with `defaultScrollPosition`.
- `message-scroller-previous-context`: a `Slider` ("Previous context peek") drives `scrollPreviousItemPeek`.
- `message-scroller-animation`: a `Select` of seven presets (fade, slide-up, slide-side, pop, spring-bounce, blur-fade, scale-fade) built on `motion/react` (`message-animations.ts`).
- `message-scroller-commands`: a `DropdownMenu` "Jump to..." that calls `useMessageScroller().scrollToMessage(id, {align:"start", behavior:"smooth"})` for each user message.
- `message-scroller-visibility`: a `HoverCard` outline that reads `useMessageScrollerVisibility().currentAnchorId` and calls `scrollToMessage`. Needs the visibility hook (see section 5).
- `message-scroller-group-chat`: a "Add Rocky / Grace" control. A join `Marker variant="separator"` is an anchored item (`scrollAnchor` on a marker).
- `message-scroller-load-history`: "Load History" button; `scrollAnchor={false}` on the separator row.
- `message-scroller-streaming`: simulated stream, `Reset`, and the same composer.

**Bubble and marker** demos are mostly static: `bubble-variants` (7 variants), `bubble-reactions` (side/align, and a destructive bubble with a "Yes, run it" button plus `toast`), `bubble-tooltip`, `bubble-popover`, `bubble-collapsible` (show more/less `Collapsible` inside `BubbleContent`), and `bubble-link-button` (three `tinted` bubbles as buttons, each firing `toast`). For the markers, `marker-icon` includes `className="flex-col"`, and `marker-link-button` mixes an `<a>` and a `<button>`.

**Attachment** demos: `attachment-demo` combines an image `AttachmentGroup` (vertical), an `uploading` card with a `Spinner` and a "Cancel upload" action, and a code file. `attachment-trigger` wraps the card in a `Dialog`, with `DialogTrigger render={<AttachmentTrigger aria-label=... />}`.

## 3. Gap table

Legend: **matches** = same demo intent is covered; **partial** = some of it; **missing** = no equivalent demo.

| shadcn example | Our closest variant | Gap | What it needs (checked against our APIs) |
|---|---|---|---|
| message-demo | message/main + marker status | partial | Composite of bubble group, footer "Delivered", and a `Marker role="status"` with `dx-shimmer` ("Oliver is typing..."). Demo-only: `Marker`, `MarkerContent`, `MessageFooter` all exist. |
| message-avatar | message/avatar | matches | none |
| message-group | message/group | matches | none |
| message-header-footer | message/header_footer | matches | none |
| message-actions | message/actions | matches | Copy/Like/Retry present. Demo-only. |
| message-attachment | message/attachment | partial | Ours has no Download action on the file card. Demo-only (`AttachmentAction` takes any `Button` props). |
| message-markdown (examples only) | none | missing | Markdown rendering. Our repo has no markdown crate and no renderer. Shadcn uses `streamdown` + `@streamdown/code`. **Needs a dependency decision** (section 5, C3). |
| bubble-demo | bubble/main + group + reactions | matches | none |
| bubble-variants | bubble/main | matches | 7 variants present. Ghost markdown copy becomes plain text (see C3). |
| bubble-alignment | bubble/alignment | matches | none |
| bubble-group-demo | bubble/group | matches | none |
| bubble-link-button | bubble/link_button | partial | Ours has two items, not a three-bubble group with `toast` feedback. Demo-only (`BubbleContent r#as`, `Toast`). |
| bubble-reactions | bubble/reactions | matches | `BubbleReactions` side/align, `Button IconXs`. Destructive "Yes, run it" text button is demo-only. |
| bubble-collapsible | none | missing | `Collapsible` / `CollapsibleTrigger` exist. Demo-only, inside `BubbleContent`. Verify the trigger can render as a `Button` link variant (not checked). |
| bubble-tooltip | none | missing | `Tooltip` exists. Demo-only: `Tooltip` around a `Button` inside `BubbleReactions`. |
| bubble-popover | none | missing | `PopoverRoot`/`PopoverTrigger`/`PopoverContent`/`PopoverHeader`/`PopoverTitle`/`PopoverDescription` exist. Demo-only. |
| bubble-markdown (examples only) | none | missing | Markdown (C3). |
| marker-demo | marker/main | matches | none |
| marker-variants | marker/variants | matches | default, separator, border. |
| marker-status | marker/status | matches | `role="status"` + `Spinner`. |
| marker-shimmer / shimmer-marker | marker/status (`dx-shimmer`) | matches | none (both are the same demo). |
| marker-separator | marker/variants | matches | none |
| marker-border | marker/variants | matches | none |
| marker-icon | marker/main | partial | Our `MarkerIcon` demos exist. The `flex-col` stacked example is missing. Demo-only (`className` on `Marker`). |
| marker-link-button | marker/link_button | matches | `Marker r#as` exists. |
| attachment-demo | attachment/image + main + states | partial | No single composite of image group + uploading card + code file. Demo-only. |
| attachment-image | attachment/image | partial | No `AttachmentTrigger` that opens each image. Demo-only: `AttachmentTrigger r#as` renders `<a href>`. |
| attachment-states | attachment/states | matches | Five states present. |
| attachment-sizes | attachment/sizes | matches | default, sm, xs. |
| attachment-group | attachment/group | partial | Ours has no per-item remove `AttachmentAction`. Demo-only. |
| attachment-trigger | attachment/trigger | partial | Ours has a trigger with `onclick` text, not a Dialog. Demo-only: `Dialog` exists. |
| message-scroller-demo | message_scroller/main | partial | Ours has streaming and anchoring, plus a Switch for auto-scroll and a `ScrollStatus` label. Missing: `Card` shell, `Empty` state, `Reset` with `Tooltip`, composer with `+` menu. Demo-only approximation. Faithful composer needs C2. |
| message-scroller-anchoring | none | missing | `ToggleGroup`/`ToggleItem` exist. Demo-only: `scroll_anchor` is a per-item prop. |
| message-scroller-group-chat | none | missing | Demo-only: a separator `Marker` as an anchored `MessageScrollerItem`. |
| message-scroller-previous-context | none | missing | `Slider` exists. Demo-only: `Slider` `value` is `ReadSignal<Option<f64>>`, mapped into the provider's `scroll_previous_item_peek: ReadSignal<f64>`. |
| message-scroller-streaming | message_scroller/main | partial | Ours simulates the stream with buttons, no composer. Demo-only. |
| message-scroller-opening-position | message_scroller/last_anchor | partial | Ours shows `LastAnchor` only. Missing: three-way `Tabs` with `key` remount. Demo-only. |
| message-scroller-load-history | message_scroller/load_history | matches | "Load earlier messages" button and start jump. |
| message-scroller-animation | none | missing | `Select` exists. Presets need entrance keyframes. Theme has `dx-fade-in/out`, `dx-pop-in/out`, `dx-drop-in/out`, `dx-edge-in/out-*`. **No** spring, blur or slide-side keyframes. Demo-only for the three that exist; C4 for the rest. |
| message-scroller-commands | none | missing | `use_message_scroller()` is exported by the wrapper, so `scroll_to_message` works. Demo-only (`DropdownMenu`). Highest value: the commands API has no demo today. |
| message-scroller-visibility | none | missing | **Needs a primitive** (`use_message_scroller_visibility`, C1). `provider scroll_margin` exists and matches shadcn's demo. |
| message-scroller-scrollable | message_scroller/main `ScrollStatus` | matches | `use_message_scroller_scrollable` exported. Shadcn's footer sentences are demo copy. |
| message-scroller-state (examples only) | message_scroller/main `ScrollStatus` | matches | Same as above. |

Counts: matches 21, partial 11, missing 11. That is 43 rows for 44 files, because `marker-shimmer` and `shimmer-marker` share one row.

## 4. Our component API vs shadcn parts (no gap unless noted)

- Present: `Message` (`align` Start/End), `MessageGroup`, `MessageAvatar`, `MessageContent`, `MessageHeader`, `MessageFooter`; `Bubble` (7 variants, `align`), `BubbleGroup`, `BubbleContent` (`r#as`), `BubbleReactions` (`side`, `align`); `Marker` (`variant`, `r#as`), `MarkerIcon`, `MarkerContent` (`dx-shimmer` class); `Attachment` (`state`, `size`, `orientation`), `AttachmentMedia` (`variant`), `AttachmentTrigger` (`r#as`), `AttachmentAction`, `AttachmentGroup`; `MessageScrollerProvider` (`auto_scroll`, `default_scroll_position`, `scroll_previous_item_peek`, `scroll_margin`, `virtualize`), `MessageScroller`, `MessageScrollerViewport` (`preserve_scroll_on_prepend`), `MessageScrollerContent`, `MessageScrollerItem` (`message_id`, `scroll_anchor`), `MessageScrollerButton` (`direction`, `behavior`), plus `use_message_scroller`, `use_message_scroller_scrollable`, and `ScrollToMessageOptions { align, behavior }`.
- Not exported from the component wrapper: `use_message_scroller_visibility` (primitive stage 2, backlog row 146).
- Per-call `scroll_margin` on `ScrollToMessageOptions` does not exist (it is provider-only). No shadcn demo uses a per-call margin, so this is API parity only.
- `InputGroupAlign` has only `Start`/`End` (`input_group/style.css` lines 71-75). Shadcn's composer footer uses `align="block-end"` (a full-width bottom row). Our `InputGroup` has no block-end addon and only a single-line `InputGroupInput`.
- No markdown renderer anywhere in the chat kit (`attachment/variants/trigger` only has the word "Markdown · 12 KB" as a file type).
- Shadcn docs say `Message` takes a `role` prop to set alignment (bubble docs, line "You can use the `role` prop"). Its API table lists only `align`. Shadcn-side doc inconsistency; ours matches the API table.

## 5. Recommendation: ordered

### A. Demo-only (existing API suffices), in order

1. **`message-scroller-commands`** ("Jump to..." dropdown, `use_message_scroller().scroll_to_message`). The commands API has zero demos today.
2. **`message-scroller-anchoring`** (`ToggleGroup`, per-role `scroll_anchor`). Teaches the role-independent anchor.
3. **`message-scroller-opening-position`** (`Tabs`, `key` remount, three `default_scroll_position` values). Only `LastAnchor` is demoed today.
4. **`message-scroller-previous-context`** (`Slider` into `scroll_previous_item_peek`). The peek prop has no demo.
5. **`message-scroller-group-chat`** (separator `Marker` as an anchored row, add-user controls).
6. **`attachment-trigger` with a real `Dialog`** and **`attachment-image`** with an `<a>` trigger per image.
7. **`bubble-tooltip`**, **`bubble-popover`**, **`bubble-collapsible`** (overlay and disclosure inside `BubbleContent`/`BubbleReactions`). Verify `CollapsibleTrigger` can render as a Button before writing.
8. **`message-demo`** composite (typing `Marker` with `dx-shimmer`, footer "Delivered") and **`marker-icon` `flex-col`**.
9. **`message-attachment`** with a Download action, and **`attachment-group`** with per-item remove actions.
10. **`bubble-link-button`** as a three-bubble group with `Toast` feedback.
11. **`message-scroller-animation`** for the three presets with theme keyframes (`dx-fade-in`, `dx-pop-in`, `dx-drop-in`). Keep it demo-scoped CSS; do not add component `@keyframes`, which the `anchored-keyframes` and `duplicate-keyframes` gates police.
12. **`message-scroller-demo`** approximation (Card shell, Empty state, Reset with Tooltip, read-only composer with inline addons). Do this after C2, or accept the approximation.

### B. Needs a component or primitive change

- **C1. `use_message_scroller_visibility`** (backlog row 146, stage 2): `currentAnchorId` and `visibleMessageIds`, IntersectionObserver-backed, pay-for-what-you-use. Unlocks `message-scroller-visibility` (HoverCard outline). Primitive change plus wrapper re-export plus JS.
- **C2. `InputGroup` block-end addon** (`InputGroupAlign::BlockEnd`, plus `.dx-input-group-addon[data-align="block-end"]`). Unlocks a faithful composer for `message-scroller-demo` and `message-scroller-streaming`. Small component change. A multi-line control is not needed for shadcn parity (their composer is a read-only span).
- **C3. Markdown rendering** for `message-markdown`, `bubble-markdown` and the ghost bubble in `bubble-variants`. This is a **dependency decision**: adding a Rust markdown crate, or accepting plain-text demos. Not decided here.
- **C4. Theme entrance keyframes** for `spring-bounce`, `blur-fade` and `slide-side`, if the owner wants all seven presets. Theme/tokens change, optional.
- **C5. Per-call `scroll_margin`** on `ScrollToMessageOptions`. No shadcn demo needs it. Low priority.

Related but not a demo gap: backlog row 148 (a "Chat" sidebar category) would move these five out of `DataDisplay`. Row 186 (utilities) documents `scroll-fade`/`shimmer` on the same chat pages.

## 6. Unverified or caveats

- Each row's "matches" compares intent and the set of parts used, read from source. Pixel and style parity was not checked (no browser run).
- The shadcn example list comes from `main` on 2026-10-14. The catalog (`dev-docs/research/shadcn-catalog-2026-10-04.md`) cites snapshot `295a1f1`. The local clone is `30a58d4`. Sampled files match, but a full diff was not run.
- The shadcn docs embed `"styleName": "base-rhea"` and import `@/styles/base-rhea/ui/*`. We have no `rhea` flavour; our stylesheets cite Nova (for example the `message_scroller/style.css` header). Treat shadcn's class names as intent, not as CSS to copy.
- No `CollapsibleTrigger`-as-Button or `DropdownMenuTrigger`-as-`Button` polymorphism was checked against our component code. The docs say the components accept `render`, ours use `r#as`. Verify before writing each demo.
