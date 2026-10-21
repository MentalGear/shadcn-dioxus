<!-- Spike report, 2026-10-21, lane/ssr-parity-spike off main 0602f7f4. Test output quoted below was produced on this branch (debug test build); scratch logs live under /home/claude/tgt and are disposable. -->

# Row 329 spike: can a native `VirtualDom` check SSR vs first-client-frame parity in `cargo test`?

**Verdict: yes, it holds up, for a defined class.** A non-wasm Dioxus 0.7.9 `VirtualDom` renders both frames of a hydrating page, including `use_effect`, `use_hook` and the signal writes the effects make, inside `cargo test`, with no renderer. It covers every component demo (503 variants) in about 9 s and 94 whole pages through the site's own `Route` router in about 48 s (debug). It catches the *"the markup is not the same before and after hydration"* class: rows 306 exactly, row 204's markup half, and it found three new instances on the current tree. It cannot see anything the browser produces (layout, measurement, `document::eval` replies, interpreter DOM writes), so rows 215 and 252 (measured sizes) and today's photo-wall `style` bug are out of its reach. It narrows the browser specs; it does not replace them.

Landed: `preview/src/ssr_parity.rs` (`#[cfg(test)] mod ssr_parity;` in `preview/src/main.rs`, `tokio` as a preview dev-dependency). 9 tests, all green.

## 1. How the two frames are produced

**Server frame.** `dioxus-server-0.7.9/src/ssr.rs:211` calls `virtual_dom.rebuild_in_place()`, then (`:216`, streaming off) `wait_for_suspense()`, then renders with a `Renderer` whose `pre_render = true` (`:442-445`). `wait_for_suspense` (`dioxus-core-0.7.9/src/virtual_dom.rs:641`) polls only tasks that run during suspense (`:667`) and never calls `pop_effect`, so **no `use_effect` runs on the server**. The test's server frame is `rebuild_in_place` + `dioxus_ssr::render` (no hydration markers, so the two frames compare byte for byte).

**Client frame.** `dioxus-web` hydrates the same first render, then its work loop calls `wait_for_work` / `render_immediate`. On a native `VirtualDom` the same driver is available without a renderer:

- `render_immediate` (`virtual_dom.rs:599`) starts with `process_events` (`:494`), which calls `poll_tasks` (`:507`) once no scope is dirty. `poll_tasks` runs the queued tasks and then pops and runs every pending effect (`:529-530`), re-checking for dirty scopes after each one, then `render_immediate` re-renders the dirtied scopes, and `finish_render` (`runtime.rs:337`) queues the next batch.
- `use_effect` (`dioxus-hooks-0.7.9/src/use_effect.rs:11`) is `queue_effect` plus a `spawn`; neither needs a renderer. A scratch component that fills a registry signal in `use_effect` and reads it in the render shows the label only in the client frame (test `a_registry_filled_by_an_effect_is_seen`), and a first-signal write in an effect (`flex-basis: 0%` -> `50%`) likewise (`a_first_signal_write_in_an_effect_is_seen`).
- **Order relative to the first paint.** Effects run after the hydration render has committed and before the browser's next frame; the writes they make are applied on the next `render_immediate`. So "server frame" is what is painted before the wasm loads and "client frame after effects" is what is painted once it has, which is exactly the pair whose difference is a repaint or a re-layout.
- Two details that cost a wrong first result. (1) The loop must run **until three consecutive `render_immediate` calls write no mutation** (`settle`, `ssr_parity.rs:72`). On a routed page the first rounds only re-run a dirty scope, write no mutation, and the effects behind them have not run yet; stopping at the first quiet round reported `/component/avatar/` as identical when 26 avatars change state. A page needs 12-66 active rounds (many small effects, one scope each); the cap is 300 and a hit fails the test as an effect loop. (2) A tokio runtime must be *entered* (`rt.enter()`, no `block_on`): `primitives/src/sleep.rs` holds tokio timers on native and panics on first poll outside a runtime.
- Context the demos need that the site's `App` provides: the i18n context (`Root`/`RouteRoot` repeat `use_init_i18n` from `main.rs`) and, for pages, a `MemoryHistory` provided as `Rc<dyn History>` (what `dioxus-server-0.7.9/src/ssr.rs:177` does) under `Router::<Route>`.

**Comparison view.** `view(html) = canonical_ids(paint_view(html))`:
- `paint_view` drops attributes that wire elements and cannot change a box (`id`, `for`, `tabindex`, `aria-controls`/`labelledby`/`describedby`/`owns`/`activedescendant`, `data-dx-motion-key`) and the `<option>` elements of the hidden native `<select>` mirror that Select keeps for form values. It keeps `class`, `style`, every `data-*`, `hidden`, `value`, `aria-label` and all text: stylesheets key on `data-*`, and an accessible name is content (row 306 was an `aria-label` too). Without the wiring filter 104 of 503 demos differ (`aria-controls` and roving `tabindex` are wired in effects on purpose); with it 43, and with the option mirror also dropped and the settle loop corrected, 35.
- `canonical_ids` renames `dxc-N` / `dxc-cN` by first appearance. The real client reuses the server's ids (`use_server_cached`, `primitives/src/lib.rs:200`); a native run allocates from the one process counter (`:178`).

## 2. Results on the current tree (main 0602f7f4 plus this test)

```
demos : 503 variants -> 35 differ, 4 cannot render (need a RouterContext: navbar x3, top_layer/main), 0 unsettled
pages : 94 routes    -> 9 differ, 0 panicked, 0 unsettled, 48.2 s (debug)
cargo test -p preview ssr_parity  ->  test result: ok. 9 passed; 0 failed; finished in 51.05s
```

The 35 differing demos and the 9 pages are pinned in `BASELINE` / `ROUTE_BASELINE`, one reasoned class each, shrink-only (a new divergence fails with its markup; an entry that stops differing fails too):

| Class | Demos | What changes between the frames |
|---|---|---|
| Avatar | 10 (avatar x5, drag_and_drop_list, empty, item x2, dropdown_menu/account_switcher) | image state `empty` -> `loading` in the first effect. The picture and fallback now share one grid cell, so no box moves (row 204). |
| Carousel | 18 | slide count read from the mounted track (`data-slide-count` 0 -> 5), previous/next `disabled` -> enabled, autoplay label "Start" -> "Stop". |
| Find layer | 1 (virtual_list/find_in_page) | an `aria-hidden`, absolutely positioned layer mounted by an effect. |
| **tag_group (new, open)** | 3 | `aria-label="Remove item "` (no label) and `disabled=true` in the server markup, `"Remove item bug"` and enabled after the effect. This is row 306's class in a component 306 did not touch; the `first-paint-text` spec did not flag it. |
| **resizable (new, open)** | 3 | panels are `flex-basis: 0%` in the server markup and take 50% / 40% in an effect: the first paint has collapsed panels and then snaps. Page level: `/component/resizable/` goes from 10 collapsed panels to 0. Not confirmed in a browser. |

Not in the table, and clean at demo and page level: Select, Combobox, Charts (the whole `/charts/` gallery and its six tabs), Calendar and DatePicker (their `internationalized` variants panicked on a missing i18n context until the test's root provided it), `/docs`, `/demos`, `/component/select/`, `/component/navbar/`.

## 3. Which historical rows it would and would not have caught

**Reproduced: row 306 (Select / Combobox no selected label), pre-fix.** A scratch worktree at `fb8c0b65^` (`f62953ad`, before `item_to_label`) with the same test file, run against the same target dir:

```
SAME select/main
DIFF select/rtl
  server: <button aria-label="Select an option" ... ><span data-placeholder=false>Select an option</span> ...
  client: <button aria-label="first" ...            ><span data-placeholder=false>first</span> ...
DIFF combobox/controlled
  server: <input ... aria-label="Controlled framework" ... value=""/>
  client: <input ... aria-label="Controlled framework" ... value="SvelteKit"/>
cargo test -p preview ssr_parity::tests::demos_paint ... FAILED (combobox/controlled named in the problem list)
```

That is the row's own two symptoms, found in seconds with no browser and no hydration. On the fixed tree both demos are identical; the same class is still present in tag_group today (table above).

| Row | Verdict | Why |
|---|---|---|
| 306 Select/Combobox label | **Caught**, reproduced above | registry filled in `use_effect`; pure Rust. |
| 204 Avatar `data-state=empty` renders image + fallback | **Markup half caught, layout half not.** The test flags `empty` -> `loading` before and after row 204's fix, because that fix was CSS (stack the two in one grid cell), so the divergence stays and sits in the baseline with that reason. It tells you the divergence exists, not that it is harmless. A box-level spec (`layout-stability`) is still what proves it. | markup differs; box effect is CSS. |
| 204 `content-visibility: auto` on masonry cards re-balancing columns | **Not caught** | CSS and layout only. |
| 215 Charts re-measured at hydration | **Not caught** | the svg's box changed because the browser measured the wrapper (`pending` -> measured). Natively no measurement arrives, so both frames are the "pending" frame. The row's second cause (header Select as wide as its placeholder, no label yet) **is** the 306 class and would be caught. The third (interior is a stand-in by design) is invisible to a native test. |
| 252 VirtualGrid 175000 px -> 175036 px | **Not caught** | the extra 36 px is the sum of *measured* rows, reported by the JS bridge (`primitives/src/virtual/bridge.rs:185`, a `document::eval` channel). Without a reply the client frame equals the server frame. Possible extension: a fake `Document` whose `eval` returns a scripted `ScrollReport` (see 4); the numbers would be the author's guess. |
| Today's photo-wall minimap `var()` shorthand lost on re-set | **Not caught** | it is the web interpreter's `setAttribute("style")` restore-longhands behaviour (`dioxus-interpreter-js`). Natively the vdom's `style` string is identical in both frames; the DOM write is the bug. `check-dynamic-style-shorthand.sh` is the right gate for it. |

Classes **catchable natively**: anything where the component's own Rust decides a different value in an effect or after a first signal write: labels and values from registries, `disabled` / `data-state` / `aria-*` flips, counts, `flex-basis`/inline-size numbers computed in Rust, elements mounted or removed by an effect (find layers, fallbacks). Classes **not catchable**: layout and box effects of CSS, anything measured (`onmounted`, ResizeObserver, `getBoundingClientRect`, scroll reports), `document::eval` replies, interpreter / web-sys DOM behaviour, stylesheet delivery order, hydration *mismatch* (the client walking server markup that disagrees with its own first render; both frames here come from the same code, so a mismatch caused by a server-only or client-only branch is invisible, see 4).

## 4. Caveats

- **Same code on both sides.** A native test renders the server and the client from one binary. A `cfg(feature = "web")` / `server_only!` / `fullstack!` arm that makes the real client render differently from the real server (75 such sites across `primitives/src` and `preview/src`) is compiled out; the `web` feature cannot run natively (wasm-bindgen imports panic off wasm32). So this checks *effects change markup*, not *the client's first render equals the server's*. `hydration-parity.spec.ts` (92 s) still owns the second.
- **`document::eval` is a no-op.** `NoOpDocument::eval` returns `EvalError::Unsupported` (`dioxus-document-0.7.9/src/document.rs:121`), 53 files use it. Anything an eval reply drives never happens. A fake `Document` provided as `Rc<dyn Document>` on the root context could script replies; not built (the scripted numbers would be invented, and the arithmetic fixes in rows 215/252 were CSS and estimate arithmetic).
- **No layout.** Nothing here knows a width, a height or a font. A divergence is a candidate, not a verdict: the baseline's reason column is where "no box moves" gets said, and it is a claim a browser spec should still back.
- **Router.** Works. Pages render through `Router::<Route>` with a `MemoryHistory` under `RouteRoot`; 94 pages, 0 panics. Block demos (`ComponentType::Block`) are skipped in the demo list (the page list covers them as routes); the four demos that need a `RouterContext` are listed in `UNRENDERABLE` and the test fails if that set changes. Only the page's own route renders: a demo variant shown in an iframe on the page is not part of that page's markup (so `dropdown_menu`'s `account_switcher` avatar differs at demo level but `/component/dropdown_menu/` is identical).
- **Ids, wiring attributes and the mirror `<select>` options are filtered by rule** (section 1). That is a judgement: a bug that only changes `aria-controls` or `tabindex` would not be seen. `aria-label`, `data-*`, `style`, `class`, text and `value` are all compared.
- **Cost.** ~9 s for demos and ~48 s for pages on a debug build, in `cargo test --workspace` and the gates' `test` arm. It adds `tokio` (already in `Cargo.lock`) to preview's dev-dependencies and compiles in the existing `cargo test` build.
- **A baseline needs upkeep.** Like the other shrink-only gates; a new effect-driven divergence is a red test that names the markup, which is the point.

## 5. What it would save, and what it should not replace

Recorded durations (`playwright/durations.json`, debug, 2 workers): `first-paint-text` 180 s, `first-paint-styles` 1086 s, `layout-stability` 819 s, `charts-hydration-stability` 262 s.

- `first-paint-text` (180 s) compares `value` / `textContent` / `aria-label` between JS-off and hydrated. The native test compares the same attributes and text between the server and the first client frame, on every demo and 94 pages, and found a case the spec missed (tag_group). It could be retired once the native test has run green for a round; the spec's only extra is real-browser hydration.
- `first-paint-styles` section 2 (JS-off vs hydrated style comparison): the native test covers the *inline `style` and class* part, not computed styles. It can drop the cases that are markup-only; the computed-style comparison stays.
- `layout-stability` cause 1 (a server render laid out from an assumption): only the markup-decided half is covered natively; boxes stay with the browser.
- `charts-hydration-stability`: not replaceable, the cause was a measurement.

So the realistic saving is the 180 s spec outright plus the markup-only slices of the others, not 900 s. The value is rather earlier and cheaper feedback (seconds, in `cargo test`, naming the markup) on the class that has recurred, and a gate for effect-written divergence that no spec gated.

## 6. Where things are

- Test: `preview/src/ssr_parity.rs`: `frames` / `route_frames` / `settle` (frames), `paint_view` / `canonical_ids` / `view` (comparison), `BASELINE`, `ROUTE_BASELINE`, `UNRENDERABLE` (pins), and the non-vacuity fixtures (`registry_label`, `render_time_label`, `effect_sized_box`).
- Wiring: `preview/src/main.rs` (`#[cfg(test)] mod ssr_parity;`), `preview/Cargo.toml` (`tokio` dev-dependency).
- Follow-ups for the owner (no backlog files were touched by this lane): fix tag_group's unlabeled remove buttons and resizable's collapsed first paint (or confirm them harmless in a browser); consider a fake `Document` for measured sizes; consider retiring `first-paint-text` after a green round.
