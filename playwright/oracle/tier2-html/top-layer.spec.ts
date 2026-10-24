/**
 * ORACLE: tier 2 (HTML) — top layer (`popover` attribute).
 *
 * Source: dev-docs/plan.md Phase 4.4, drawing on the WHATWG HTML Living
 * Standard's popover chapter:
 *   - The `popover` attribute, its `auto`/`manual` states, and the implied
 *     top-layer promotion:
 *     https://html.spec.whatwg.org/multipage/popover.html#the-popover-attribute
 *   - Light dismiss (Escape, outside pointerdown, closing sibling `auto`
 *     popovers) for `popover="auto"`:
 *     https://html.spec.whatwg.org/multipage/popover.html#popover-light-dismiss
 *   - `showPopover()`/`hidePopover()` and the `toggle` event fired on every
 *     state change (browser- or script-driven alike):
 *     https://html.spec.whatwg.org/multipage/popover.html#dom-showpopover
 *     https://html.spec.whatwg.org/multipage/popover.html#dom-hidepopover
 *   - The top layer itself (a rendering surface above the document that
 *     escapes ancestor overflow/transform/stacking contexts):
 *     https://html.spec.whatwg.org/multipage/rendering.html#the-top-layer
 *
 * Fixture: preview/src/components/top_layer/component.rs (`TopLayerFixture`),
 * served at /component/?name=top_layer. Every library control there is
 * paired with a native reference control — a plain `<div popover="auto">`
 * shown by a `<button popovertarget>` — the browser's own implementation of
 * the identical WHATWG HTML feature, with zero Dioxus involvement.
 *
 * Calibration (dev-docs/conformance-harness.md, "Calibration"): every rule below
 * runs against the native reference first (prefixed CALIBRATION: in the
 * test name) and the library component second. A CALIBRATION failure means
 * the *test* is wrong, not the component.
 *
 * Per-component dismissal semantics (dev-docs/plan.md Phase 4.4's own
 * requirement to "document the choice per component"):
 *   - `Tooltip`/`HoverCard` render with `popover="manual"` on the web arm.
 *     Both already own their entire open/close lifecycle through
 *     hover/focus (and, for Tooltip, an explicit Escape handler on the
 *     trigger) — MDN's own naming ("hover card") does not imply the light
 *     dismiss a click-triggered popover has, and `auto`'s outside-pointerdown
 *     dismissal would race that existing lifecycle rather than usefully
 *     extend it. So rules 2 ("light dismiss") and 3 ("Escape") below apply
 *     only to the non-modal `Popover` arm (`popover="auto"`) and its native
 *     reference — see primitives/src/top_layer.rs's `PopoverKind` doc for
 *     the full reasoning.
 *   - The non-modal `Popover` arm renders as `<dialog popover="auto">`,
 *     getting light dismiss for free from the platform; this file's rules
 *     2-3 are exactly what checks that the WHATWG-mandated browser behaviour
 *     (`toggle` event) still lands back in the Rust `open` signal, per
 *     primitives/src/top_layer.rs's `use_popover_sync` (the same defect
 *     class documented for a one-way `<dialog>` binding in
 *     dev-docs/reference/platform-rationale.md Caveat 1).
 *   - Rule 1 ("clipping escape") and rule 4 ("stacking") are top-layer
 *     properties every promoted element gets regardless of its
 *     `popover="auto"|"manual"` kind, so all three components (Tooltip,
 *     HoverCard, Popover) are checked for rule 1; rule 4 uses `Popover`
 *     (the click-triggered case is the easiest to drive deterministically).
 *   - `ContextMenu`/`Menubar` (`dev-docs/plan.md` Phase
 *     5.2's point-anchor case is explicitly out of scope for this file):
 *     `ContextMenu` also renders `popover="manual"` on the web arm, for a
 *     reason distinct from Tooltip/HoverCard's: it opens at a raw click
 *     point with no persistent trigger to key an outside-vs-internal
 *     dismissal distinction off of the way `auto`'s native light dismiss
 *     would need, so it keeps its own `use_outside_dismiss` +
 *     root `Escape` handler (see
 *     `primitives/src/context_menu.rs`'s `ContextMenuContentRendered` doc
 *     for the full reasoning and the focus-restore risk this avoids).
 *     `Menubar` menus render `popover="auto"`, anchored to their own
 *     trigger like `DropdownMenu` -- Rule 1 (clipping) and Rule 9
 *     (scroll behavior, below) cover both; rules 2/3 (light dismiss/Escape)
 *     stay scoped to `Popover` as documented above, since `ContextMenu`'s
 *     `manual` mode never light-dismisses and `Menubar`'s Escape handling
 *     is APG-specific (per-trigger refocus) rather than a generic "does the
 *     Rust signal learn about a native close" check those two rules make.
 *   - `Select`'s listbox renders
 *     `popover="auto"` on the web arm (`select/components/list.rs`'s
 *     `SelectListRendered`) -- its own more precise
 *     blur-driven dismissal/focus-restore logic stays the primary
 *     mechanism, with native light dismiss layered on as a backstop; see
 *     that component's own doc for the full reasoning. `Combobox`'s listbox
 *     renders `popover="manual"` instead -- `auto` breaks
 *     `combobox.spec.ts`'s "dynamic option
 *     removal" case, where an external control button (with its own
 *     `prevent_default()`-guarded `pointerdown`, specifically so it does
 *     not close the combobox) gets light-dismissed anyway, since WHATWG
 *     light dismiss classifies any outside pointerdown as "outside" full
 *     stop, `prevent_default()` or not -- see
 *     `combobox/components/list.rs`'s `ComboboxListRendered` doc for the
 *     complete account. Both are exercised only by Rule 1 (clipping) below,
 *     not rules 2/3, for the same reason `ContextMenu`/`Menubar` aren't:
 *     each keeps its own more precise dismissal path as the actual
 *     mechanism, with `auto` (where used) only ever a backstop. `Toast`'s
 *     region renders `popover="manual"` (`toast.rs`'s `ToastRegionRendered`)
 *     -- it has no open/close lifecycle and nothing to anchor to, so only
 *     the stacking property (Rule 10, below) applies to it; Rule 1
 *     does not, since a toast is not clipped by an ancestor the way a
 *     trigger-anchored popup is (it is viewport-region positioned, not
 *     positioned relative to any particular trigger element).
 *   - `NavigationMenu`'s panel (dev-docs/backlog.md row 72) also
 *     renders `popover="manual"`, for the same reason as Tooltip/HoverCard/
 *     ContextMenu/Menubar above: it owns its entire open/close lifecycle
 *     (hover-intent timers, a focus-leave debounce, its own Escape handler
 *     with APG-style per-trigger refocus) -- see
 *     `primitives/src/navigation_menu.rs`'s module doc, "Top layer", for the
 *     full reasoning. It implements a *different* APG pattern from `Navbar`
 *     (Disclosure Navigation, not Menu/Menubar), but shares the identical
 *     top-layer/anchor-positioning engine, so it is checked wherever Navbar
 *     is: Rule 1 (clipping), the axe sweep, Rule 5 (bottom-edge flip -- no
 *     `side` prop, same as Navbar), Rule 11 (self-overlap/iOS keyboard, both
 *     `ANCHORED_OVERLAYS` and `CONFORMING_CASES`), Rule 13 (fit-content
 *     width), and Rule 14 (aria-labelledby resync) -- the identical rule set
 *     Navbar carries, for the identical reason (both are hover-opened,
 *     trigger-anchored `use_anchor_position_fallback` consumers). Not
 *     checked by rules 2/3 (light dismiss/Escape, `manual`-mode scoped to
 *     `Popover` alone) or rules 4/6/8/9/12/15 (single-subject or
 *     `side`-prop-dependent rules Navbar is equally excluded from, for the
 *     same reasons already documented on those rules).
 *
 * Rules implemented (dev-docs/plan.md Phase 4.4):
 *   1. Clipping escape — component content inside an ancestor with
 *      `overflow: hidden` + `transform` + a height shorter than the
 *      content renders fully visible outside the clip.
 *   2. Light dismiss for `popover="auto"` — click outside closes it AND
 *      Rust state (`data-state` on the `PopoverRoot`) syncs.
 *   3. Escape closes `popover="auto"` AND Rust state syncs.
 *   4. Top-layer stacking — a popover opened after a high-z-index sibling
 *      renders above it.
 *   5. Near-viewport-edge flip — see "Rules 5-7" below.
 *
 * ## Rules 5-7 — CSS Anchor Positioning flip (dev-docs/backlog.md row 10)
 *
 * Unlike every rule above, this trio's citation is the **W3C CSS Anchor
 * Positioning** spec, not WHATWG HTML — this tier's "standard" principle
 * (dev-docs/conformance-harness.md) is satisfied by that spec instead:
 *   - `position-try-fallbacks` and the `flip-block`/`flip-inline` "existing
 *     tactics":
 *     <https://www.w3.org/TR/css-anchor-position-1/#fallback-var>
 *     <https://www.w3.org/TR/css-anchor-position-1/#accepted-existing-tactics>
 *
 * It lives in this file anyway (rather than a new spec file) because it
 * exercises the exact same fixture family (anchor-positioned `popover`
 * content) and the exact same per-component dismissal split documented
 * above — a sibling file would just re-import all of that context.
 *
 * Fixture: the "Near-viewport-edge flip" section of `TopLayerFixture`
 * (`preview/src/components/top_layer/component.rs`) — a bottom-edge row and
 * a right-edge column, each pinned by `position: fixed` a few pixels from
 * that viewport edge (deterministic regardless of page height/scroll, same
 * reasoning as the existing stacking-sibling fixture). Every trigger in a
 * row/column requests the side pointing *off* that viewport edge
 * (`side="bottom"` a few px from the bottom; `side="right"` a few px from
 * the right), so the preferred placement cannot fit and only a flip lands
 * it on-screen. Each row/column also carries a native `<div popover>` +
 * `position-try-fallbacks` reference (CALIBRATION) — the browser's own
 * implementation of the identical spec feature, no Dioxus code involved.
 *
 * Rule 5 (block-axis flip): a bottom-edge trigger with `side="bottom"`
 * renders its content fully inside the viewport, above the trigger.
 * Rule 6 (inline-axis flip): a right-edge trigger with `side="right"`
 * renders its content fully inside the viewport, to the left of the
 * trigger. All three anchor-positioned components (Tooltip, HoverCard,
 * non-modal Popover) support every `ContentSide` value, so the inline axis
 * applies to all three — no side is skipped here.
 * Rule 7 (no spurious flip): a normal, mid-viewport placement (the existing
 * "Stacking popover" from the section above, `side="bottom"` far from any
 * edge) renders unflipped, below its trigger — this is the guard
 * for `primitives/src/top_layer.rs`'s `use_anchor_position_fallback`: since
 * that hook's JS fallback accepts *either* the primary or the flipped
 * placement as contract-legal (see its doc), this rule confirms it isn't
 * accepting a flip that never should have happened in the common case.
 */

import { test, expect } from "../../fixtures";
import { type Page } from "@playwright/test";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "../../axe";
import { BASE_URL } from "../../base-url";
import { gotoHydrated } from "../../hydration";
import { fixedViewportBox } from "../../viewport-box";
import { readAtRest } from "../../animations";
import { demoRole } from "../../demo-frame";

const NAV_TIMEOUT = 20 * 60 * 1000; // first run compiles the app

// dev-docs/backlog.md row 109: this suite clicks/hovers triggers right
// after navigating, on the SSG lane -- `gotoHydrated` so those interactions
// can't land before hydration attaches listeners.
const gotoFixture = (page: Page) =>
  gotoHydrated(page, `${BASE_URL}/component/top_layer/`, {
    timeout: NAV_TIMEOUT,
  });

/**
 * The real discriminating test for "does this content escape the clip,"
 * not just "is its geometric box taller than the clip." A normal (non-top-
 * layer) DOM child's `getBoundingClientRect()` reports its full laid-out
 * size regardless of an ancestor's `overflow: hidden` -- clipping only
 * affects *paint*, not layout geometry -- so a box-size comparison alone
 * cannot tell a genuinely-escaped element from one that is simply present
 * but invisible past the clip. `document.elementFromPoint()` at a
 * coordinate inside the content's own box but outside the clip ancestor's
 * box resolves to whatever is actually *painted* there: the content itself
 * if it escaped (top layer), or whatever sits behind/around the clip box
 * if it did not (the clipped behaviour this rule exists to
 * catch).
 *
 * Candidate probe points are the content box's four corners (inset 2px, or by
 * the content's computed border radius if that is larger, so the probe lands
 * inside a rounded corner rather than on the notch cut out of it) plus its center, not just "just past the ancestor's edge": a top-layer
 * element's landing spot varies by engine/positioning support (an
 * anchor-positioned popover lands right next to its trigger, which sits
 * inside the clip ancestor; the native `<div popover>` reference here has
 * no anchor-positioning applied at all, so the UA default
 * `position: fixed; inset: 0; margin: auto` centers it in the viewport --
 * nowhere near the clip ancestor). Whichever candidate point falls outside
 * the ancestor's box *and inside the actual browser viewport* is used; if
 * content and ancestor don't overlap at all, every on-screen candidate
 * qualifies.
 *
 * The viewport bound matters: `element.getBounding
 * ClientRect()` reports a box's full geometry regardless of whether any of
 * it is actually on-screen, but `document.elementFromPoint()` is a
 * viewport-relative API that returns `null` for any coordinate outside
 * `[0, innerWidth) x [0, innerHeight)` -- always, regardless of what
 * top-layer content would otherwise paint there. `use_anchor_position_
 * fallback` (primitives/src/top_layer.rs) can legitimately compute a
 * position
 * that runs partly off-screen for this fixture's unstyled, unconstrained-
 * width tooltip content -- a real, on-screen escape from the clip ancestor
 * either way, just not always at the *first* candidate in box-corner
 * order. Without this bound, a build could flip between candidate corners
 * that are on- vs off-screen for reasons unrelated to top-layer escape
 * (timing/measurement noise in exactly where the unconstrained-width
 * content lands), intermittently reporting a real escape as a false
 * negative merely because `elementFromPoint` was asked about a pixel that
 * does not exist.
 */
async function escapesClip(
  page: Page,
  contentSelector: string,
  ancestorSelector: string,
): Promise<{ escapes: boolean; reason?: string; x?: number; y?: number; hit?: string | null }> {
  return page.evaluate(
    async ({ contentSelector, ancestorSelector }) => {
      const content = document.querySelector(contentSelector) as HTMLElement | null;
      const ancestor = document.querySelector(ancestorSelector) as HTMLElement | null;
      if (!content || !ancestor) {
        return { escapes: false, reason: `missing element(s): content=${!!content} ancestor=${!!ancestor}` };
      }
      // Wait out any entrance animation on `content` itself before probing
      // -- same idiom `toast.spec.ts` uses. A probe issued mid-animation
      // makes `document.elementFromPoint` at the content's own top-left
      // corner return the page's "Clipping escape" `<h2>` instead of the
      // (correctly top-layered, `position:fixed`, `z-index:1000`) content
      // (NavigationMenu's slow, genuinely running entrance animation,
      // `--dx-motion-duration-slow`, shows it first; row 72). Every one of
      // this describe block's 11 `escapesClip` calls shares this "probe
      // immediately after hover/click, before any entrance animation
      // settles" shape, so the wait lives in the shared helper rather than
      // at one call site.
      await Promise.all(
        content.getAnimations({ subtree: true }).map((a) => a.finished.catch(() => undefined)),
      );
      const c = content.getBoundingClientRect(); // rest-ok: settled by the getAnimations().finished await directly above
      const a = ancestor.getBoundingClientRect(); // rest-ok: the clipping ancestor, not an overlay
      // Inset each corner probe by at least the content's own border radius. A box's
      // corner is not painted where its rounded border has cut it away: a probe 2px
      // inside the corner of a 10px-radius panel (Nova's `rounded-lg`, the popover
      // and every menu panel) lands on whatever is BEHIND that notch, so the content
      // is reported as clipped when it merely has a rounded corner. A point `radius`
      // in from both edges is the arc's own centre, always inside the painted shape,
      // so the probe still asks the only question this rule has: "does the content
      // paint here, outside the clip ancestor's box?".
      const cs = getComputedStyle(content);
      const radius = Math.max(
        ...["borderTopLeftRadius", "borderTopRightRadius", "borderBottomLeftRadius", "borderBottomRightRadius"].map(
          (p) => parseFloat((cs as unknown as Record<string, string>)[p]) || 0,
        ),
      );
      const inset = Math.max(2, radius);
      const candidates: [number, number][] = [
        [c.left + inset, c.top + inset],
        [c.right - inset, c.top + inset],
        [c.left + inset, c.bottom - inset],
        [c.right - inset, c.bottom - inset],
        [c.left + c.width / 2, c.top + c.height / 2],
      ];
      const outsideAncestor = (x: number, y: number) =>
        x < a.left || x > a.right || y < a.top || y > a.bottom;
      const onScreen = (x: number, y: number) =>
        x >= 0 && x < window.innerWidth && y >= 0 && y < window.innerHeight;
      const probe = candidates.find(([x, y]) => outsideAncestor(x, y) && onScreen(x, y));
      if (!probe) {
        return {
          escapes: false,
          reason: `no on-screen candidate in content's box (${JSON.stringify(c)}) falls outside the ancestor's box (${JSON.stringify(a)}) within the viewport (${window.innerWidth}x${window.innerHeight}) -- cannot distinguish escape from containment this way`,
        };
      }
      const [x, y] = probe;
      const hit = document.elementFromPoint(x, y);
      const escapes = hit === content || content.contains(hit);
      return {
        escapes,
        x,
        y,
        hit: hit ? `${hit.tagName}#${hit.id || "(no id)"}` : null,
      };
    },
    { contentSelector, ancestorSelector },
  );
}

test.describe("Rule 1 — clipping escape (an ancestor with overflow:hidden + transform + a height shorter than the content must not clip it)", () => {
  test("CALIBRATION: native <div popover=auto> escapes the clip", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-native-trigger").click();
    await expect(page.locator("#clip-native-content")).toBeVisible();
    const result = await escapesClip(page, "#clip-native-content", "#clip-box");
    expect(result.escapes, JSON.stringify(result)).toBe(true);
  });

  test("Tooltip content escapes the clip", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-tooltip-trigger").hover();
    await expect(page.locator("#clip-tooltip-content")).toBeVisible();
    const result = await escapesClip(page, "#clip-tooltip-content", "#clip-box");
    expect(result.escapes, JSON.stringify(result)).toBe(true);
  });

  test("HoverCard content escapes the clip", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-hovercard-trigger").hover();
    await expect(page.locator("#clip-hovercard-content")).toBeVisible();
    const result = await escapesClip(page, "#clip-hovercard-content", "#clip-box");
    expect(result.escapes, JSON.stringify(result)).toBe(true);
  });

  test("Popover (non-modal) content escapes the clip", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-popover-trigger").click();
    await expect(page.locator("#clip-popover-content")).toBeVisible();
    const result = await escapesClip(page, "#clip-popover-content", "#clip-box");
    expect(result.escapes, JSON.stringify(result)).toBe(true);
  });

  // dev-docs/backlog.md item 2: DropdownMenu's web arm is
  // `popover="auto"` (`DropdownMenuContentRendered`, `dropdown_menu.rs`).
  // A plain-div `DropdownMenuContent` would clip at the 60px ancestor exactly
  // like Tooltip/HoverCard/Popover would without the top layer (Phase 4.4).
  test("DropdownMenu content escapes the clip", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-dropdown-menu-trigger").click();
    await expect(page.locator("#clip-dropdown-menu-content")).toBeVisible();
    const result = await escapesClip(page, "#clip-dropdown-menu-content", "#clip-box");
    expect(result.escapes, JSON.stringify(result)).toBe(true);
  });

  // ContextMenu's web arm is `popover="manual"`
  // (`ContextMenuContentRendered`, `context_menu.rs`). A plain
  // `position: fixed` div would clip at the 60px ancestor like the others --
  // a `transform`-ed ancestor becomes the containing block for a
  // `position: fixed` descendant, per ordinary CSS, unless the content is
  // in the top layer.
  test("ContextMenu content escapes the clip", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-context-menu-trigger").click({ button: "right" });
    await expect(page.locator("#clip-context-menu-content")).toBeVisible();
    const result = await escapesClip(page, "#clip-context-menu-content", "#clip-box");
    expect(result.escapes, JSON.stringify(result)).toBe(true);
  });

  // Menubar menus' web arm is `popover="auto"`, anchored to their own
  // trigger (`MenubarContentRendered`, `menubar.rs`). A plain
  // `position: absolute` div would clip at the 60px ancestor exactly like
  // DropdownMenu's equivalent shape would.
  test("Menubar content escapes the clip", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-menubar-trigger").click();
    await expect(page.locator("#clip-menubar-content")).toBeVisible();
    const result = await escapesClip(page, "#clip-menubar-content", "#clip-box");
    expect(result.escapes, JSON.stringify(result)).toBe(true);
  });

  // Select's web arm is `popover="auto"` (`SelectListRendered`,
  // `select/components/list.rs`). Not portal'd -- `SelectList` renders in
  // place in the tree, so a plain `div` would clip at the 60px ancestor like
  // every other listbox/menu here.
  test("Select listbox escapes the clip", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-select-trigger").click();
    await expect(page.locator("#clip-select-content")).toBeVisible();
    const result = await escapesClip(page, "#clip-select-content", "#clip-box");
    expect(result.escapes, JSON.stringify(result)).toBe(true);
  });

  // Combobox's web arm is `popover="manual"` (`ComboboxListRendered`,
  // `combobox/components/list.rs` -- see that component's doc for why
  // `manual`, not `auto`); a plain `div` would clip at the 60px ancestor
  // like the others. Opened via keyboard (ArrowDown) rather than a click, since a click on
  // the input alone does not open this popup.
  test("Combobox listbox escapes the clip", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-combobox-trigger").click();
    await page.keyboard.press("ArrowDown");
    await expect(page.locator("#clip-combobox-content")).toBeVisible();
    const result = await escapesClip(page, "#clip-combobox-content", "#clip-box");
    expect(result.escapes, JSON.stringify(result)).toBe(true);
  });

  // `NavbarNav`'s web arm is `popover="auto"` (`NavbarContentRendered`,
  // `navbar.rs`) on the top-layer engine, so it escapes the 60px ancestor.
  // A plain `position: absolute`-only div would clip there exactly like
  // `Menubar`'s equivalent shape does.
  test("Navbar content escapes the clip", async ({ page }) => {
    await gotoFixture(page);
    // `.hover()`, not `.click()` -- `navbar.spec.ts`'s own suite opens this
    // exact way (`NavbarNav`'s `onmouseenter`); a plain `.click()` also
    // synthesizes the hover Playwright performs to reach the element first,
    // which opens it, immediately followed by the click itself
    // toggling it back closed (`NavbarTrigger`'s own toggle-on-click)
    // -- an interaction of `NavbarTrigger` itself, independent of the
    // popover engine.
    await page.locator("#clip-navbar-trigger").hover();
    await expect(page.locator("#clip-navbar-content")).toBeVisible();
    const result = await escapesClip(page, "#clip-navbar-content", "#clip-box");
    expect(result.escapes, JSON.stringify(result)).toBe(true);
  });

  // dev-docs/backlog.md row 72: `NavigationMenuContent`'s web arm is
  // `popover="manual"`, anchored to its own trigger
  // (`NavigationMenuContentRendered`, `navigation_menu.rs`) -- the same
  // top-layer engine `NavbarContent` uses (`dx-anchor-navigation-menu`,
  // already wired alongside `dx-anchor-navbar` in `top_layer.rs`'s shared
  // anchor-positioning stylesheet). `NavigationMenu` implements a
  // *different* APG pattern from `Navbar` (Disclosure Navigation, not
  // Menu/Menubar -- see `primitives/src/navigation_menu.rs`'s module doc),
  // but shares the same hover/click-opened, trigger-anchored top-layer
  // shape this rule checks, so it gets an identical clip-* fixture
  // instance and open step to the Navbar case just above.
  test("NavigationMenu content escapes the clip", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-navigation-menu-trigger").hover();
    await expect(page.locator("#clip-navigation-menu-content")).toBeVisible();
    const result = await escapesClip(page, "#clip-navigation-menu-content", "#clip-box");
    expect(result.escapes, JSON.stringify(result)).toBe(true);
  });
});

/**
 * axe (static rules) — every anchored-overlay component on this fixture,
 * scanned with its content open. No component spec reaches this fixture
 * (each has its own, real-demo page instead); this is the state no other
 * spec covers, per dev-docs/backlog.md row 34's own remit. Reuses exactly the
 * open steps Rule 1 above already established for each component.
 */
test.describe("axe: every overlay open (top-layer fixture)", () => {
  test("Tooltip content open has no automatically detectable a11y issues", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-tooltip-trigger").hover();
    await expect(page.locator("#clip-tooltip-content")).toBeVisible();
    await expectNoAxeViolations(page, "top-layer fixture: Tooltip open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test("HoverCard content open has no automatically detectable a11y issues", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-hovercard-trigger").hover();
    await expect(page.locator("#clip-hovercard-content")).toBeVisible();
    await expectNoAxeViolations(page, "top-layer fixture: HoverCard open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test("Popover (non-modal) content open has no automatically detectable a11y issues", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-popover-trigger").click();
    await expect(page.locator("#clip-popover-content")).toBeVisible();
    await expectNoAxeViolations(page, "top-layer fixture: Popover open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test("DropdownMenu content open has no automatically detectable a11y issues", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-dropdown-menu-trigger").click();
    await expect(page.locator("#clip-dropdown-menu-content")).toBeVisible();
    await expectNoAxeViolations(page, "top-layer fixture: DropdownMenu open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test("ContextMenu content open has no automatically detectable a11y issues", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-context-menu-trigger").click({ button: "right" });
    await expect(page.locator("#clip-context-menu-content")).toBeVisible();
    await expectNoAxeViolations(page, "top-layer fixture: ContextMenu open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test("Menubar content open has no automatically detectable a11y issues", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-menubar-trigger").click();
    await expect(page.locator("#clip-menubar-content")).toBeVisible();
    await expectNoAxeViolations(page, "top-layer fixture: Menubar open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  // `SelectList`'s default `aria-labelledby` points at
  // `ctx.selectable.trigger_id`'s value, which must match this fixture's
  // `SelectTrigger { id: "clip-select-trigger", ... }`: `merge_attributes`'s
  // caller-wins rule overrides the *rendered* DOM id with the fixture's own,
  // so `SelectTrigger` feeds `ctx.selectable.trigger_id` straight into
  // `use_id_or` (`select/components/trigger.rs`) to keep the listbox's
  // reference from dangling (dev-docs/backlog.md row 36). See Rule 14
  // below for the direct (non-axe) regression guard on the resolved
  // reference itself, across every trigger that construction covers.
  test("Select listbox open has no automatically detectable a11y issues", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-select-trigger").click();
    await expect(page.locator("#clip-select-content")).toBeVisible();
    await expectNoAxeViolations(page, "top-layer fixture: Select open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test("Combobox listbox open has no automatically detectable a11y issues", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-combobox-trigger").click();
    await page.keyboard.press("ArrowDown");
    await expect(page.locator("#clip-combobox-content")).toBeVisible();
    await expectNoAxeViolations(page, "top-layer fixture: Combobox open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  // Navbar's `popover="auto"` content.
  test("Navbar content open has no automatically detectable a11y issues", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-navbar-trigger").hover();
    await expect(page.locator("#clip-navbar-content")).toBeVisible();
    await expectNoAxeViolations(page, "top-layer fixture: Navbar open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  // dev-docs/backlog.md row 72: the NavigationMenu oracle coverage above.
  test("NavigationMenu content open has no automatically detectable a11y issues", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#clip-navigation-menu-trigger").hover();
    await expect(page.locator("#clip-navigation-menu-content")).toBeVisible();
    await expectNoAxeViolations(page, "top-layer fixture: NavigationMenu open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

/**
 * Rule 10 — Toast top-layer stacking.
 * `ToastProvider`'s region has no open/close lifecycle and never anchors to
 * a trigger (see `primitives/src/toast.rs`'s `ToastRegionRendered` doc for
 * why it renders `popover="manual"` rather than `auto`) -- the property
 * the top layer buys it is exactly Rule 4's: painting above a competing
 * high-`z-index` sibling regardless of DOM/stacking-context position. The
 * fixture pins a small (40x20px) high-`z-index` sibling and the toast
 * region to the same `position: fixed` bottom-right viewport corner.
 *
 * No "before any toast exists" calibration half here, unlike every other
 * rule in this file -- the premise doesn't hold for this fixture: the region itself is mounted (and, being
 * `popover="manual"`, already `:popover-open`) from page load, before any
 * toast is ever added, and even with zero toasts inside it the UA popover
 * stylesheet's own `border`/`padding` defaults (this raw, unstyled
 * `dioxus_primitives::toast::ToastProvider` carries none of the real
 * component's own border/padding reset -- see
 * `preview/src/components/toast/style.css`'s `.dx-toast-container[popover]`,
 * which only applies to the *styled* wrapper) already give the empty
 * region a real, nonzero-area box at that exact corner -- so the sibling
 * is already covered before the click, not just after. That is itself
 * further (if incidental) evidence the top-layer promotion holds for the region continuously, not only while it holds a
 * toast -- but it means the meaningful check is simply "the sibling is not
 * what paints at that point once a toast exists," verified directly below
 * instead of via a before/after diff.
 */
test.describe("Rule 10 — Toast top-layer stacking (a toast renders above a maximal-z-index sibling)", () => {
  test("Toast renders above the high-z-index sibling", async ({ page }) => {
    await gotoFixture(page);
    // The sibling (and the toast region itself) are `position: fixed` to
    // the bottom-right viewport corner -- always on-screen regardless of
    // scroll, so no `scrollIntoViewIfNeeded` is needed for either of them
    // (see `preview/src/components/top_layer/component.rs`'s comment on
    // this section for why `fixed`, not `absolute`, is load-bearing here).
    // The *trigger button* is a normal in-flow element well down this
    // fixture page, though, and does need scrolling into view to be
    // clickable below.
    await page.locator("#toast-stack-trigger").scrollIntoViewIfNeeded();

    await page.locator("#toast-stack-trigger").click();
    await expect(page.getByRole("alertdialog")).toBeVisible();

    const hit = await page.evaluate(() => {
      const sibling = document.getElementById("toast-stack-sibling")!;
      const r = sibling.getBoundingClientRect(); // rest-ok: a page sibling under the overlay, not an animated surface
      const el = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
      if (!el) return { tag: null, insideRegion: false };
      return { tag: `${el.tagName}#${el.id || "(no id)"}`, insideRegion: !!el.closest('[role="region"]') };
    });
    expect(hit.tag, "expected the toast region to paint above the high-z-index sibling").not.toBe(
      "DIV#toast-stack-sibling",
    );
    expect(hit.insideRegion, `expected the hit (${hit.tag}) to land inside the toast region`).toBe(true);
  });
});

test.describe("Rule 2 — light dismiss for popover=auto (click outside closes it, and Rust state syncs)", () => {
  test("CALIBRATION: native <div popover=auto> closes on outside click", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#stack-native-trigger").evaluate((el) => (el as HTMLElement).click());
    await expect(page.locator("#stack-native-content")).toBeVisible();
    await page.locator("#outside-click-target").click();
    await expect(page.locator("#stack-native-content")).toBeHidden();
  });

  test("Popover (non-modal): click outside closes it and syncs data-state", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#stack-popover-trigger").evaluate((el) => (el as HTMLElement).click());
    await expect(page.locator("#stack-popover-content")).toBeVisible();
    await expect(page.locator("#stack-popover-root")).toHaveAttribute("data-state", "open");

    await page.locator("#outside-click-target").click();

    // The browser's own light dismiss hides the element; the Rust signal
    // must independently learn about it (primitives/src/top_layer.rs
    // `use_popover_sync`'s `toggle` listener) -- assert both, since a
    // stranded signal (the exact defect class dev-docs/reference/platform-rationale.md
    // Caveat 1 documents for <dialog>) would leave data-state="open" even
    // though the content is gone.
    await expect(page.locator("#stack-popover-content")).toBeHidden();
    await expect(page.locator("#stack-popover-root")).toHaveAttribute("data-state", "closed");
  });
});

test.describe("Rule 3 — Escape closes popover=auto, and Rust state syncs", () => {
  test("CALIBRATION: native <div popover=auto> closes on Escape", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#stack-native-trigger").evaluate((el) => (el as HTMLElement).click());
    await expect(page.locator("#stack-native-content")).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.locator("#stack-native-content")).toBeHidden();
  });

  test("Popover (non-modal): Escape closes it and syncs data-state", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#stack-popover-trigger").evaluate((el) => (el as HTMLElement).click());
    await expect(page.locator("#stack-popover-content")).toBeVisible();
    await expect(page.locator("#stack-popover-root")).toHaveAttribute("data-state", "open");

    await page.keyboard.press("Escape");

    await expect(page.locator("#stack-popover-content")).toBeHidden();
    await expect(page.locator("#stack-popover-root")).toHaveAttribute("data-state", "closed");
  });
});

test.describe("Rule 4 — top-layer stacking (a popover opened after a high-z-index sibling renders above it)", () => {
  // Scroll the section into view *before* opening (the stacking section is
  // in-flow -- see `preview/src/components/top_layer/style.css`'s comment on
  // `.dx-top-layer-stack-sibling` -- so it can render below the fold),
  // not after: opening first and scrolling afterward is exactly what Rule 8
  // below exercises deliberately, and doing it here too would make this
  // rule's assertion race the fallback's rAF-throttled re-measure for no
  // reason this rule needs to take on -- scroll-then-open is
  // deterministic where the reverse is not. `#stack-sibling` is used (not the trigger) because its box is
  // taller than the trigger row's, so scrolling it fully into view also
  // covers the row.
  test("CALIBRATION: native <div popover=auto> renders above the high-z-index sibling", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#stack-sibling").scrollIntoViewIfNeeded();
    await page.locator("#stack-native-trigger").evaluate((el) => (el as HTMLElement).click());
    await expect(page.locator("#stack-native-content")).toBeVisible();

    const hit = await page.evaluate(() => {
      const sibling = document.getElementById("stack-sibling")!;
      const r = sibling.getBoundingClientRect(); // rest-ok: a page sibling under the overlay, not an animated surface
      const el = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
      return el ? `${el.tagName}#${el.id || "(no id)"}` : null;
    });
    expect(hit).toBe("DIV#stack-native-content");
  });

  test("Popover (non-modal) renders above the high-z-index sibling", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#stack-sibling").scrollIntoViewIfNeeded();
    await page.locator("#stack-popover-trigger").evaluate((el) => (el as HTMLElement).click());
    await expect(page.locator("#stack-popover-content")).toBeVisible();

    const hit = await page.evaluate(() => {
      const sibling = document.getElementById("stack-sibling")!;
      const r = sibling.getBoundingClientRect(); // rest-ok: a page sibling under the overlay, not an animated surface
      const el = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
      return el ? `${el.tagName}#${el.id || "(no id)"}` : null;
    });
    expect(hit).toBe("DIALOG#stack-popover-content");
  });
});

/**
 * Rects and viewport size for the flip rules below. `getBoundingClientRect`
 * is the same viewport-relative measurement `escapesClip` above already
 * relies on for the same reason: layout-box geometry, unaffected by paint
 * effects, and directly comparable to `window.inner{Width,Height}`.
 */
async function rectOf(page: Page, selector: string) {
  return readAtRest(page.locator(selector), (el) => {
    const r = (el as HTMLElement).getBoundingClientRect();
    return { top: r.top, left: r.left, right: r.right, bottom: r.bottom, width: r.width, height: r.height };
  });
}

async function viewportSize(page: Page) {
  // The fixed containing block, not `innerWidth` (gutter-inclusive under `scrollbar-gutter: stable`).
  const { width, height } = await fixedViewportBox(page);
  return { width, height };
}

// A 1px tolerance throughout the flip rules below for the same
// rounding/subpixel reason `use_anchor_position_fallback`'s own >2px
// tolerance exists (primitives/src/top_layer.rs) -- comparisons here are
// looser (1px) since they only need "fully inside the viewport"/"on the
// correct side of the trigger," not exact pixel equality to a formula.
const EDGE_TOLERANCE = 1;

test.describe("Rule 5 — block-axis flip: a bottom-edge trigger with side=\"bottom\" renders its content fully inside the viewport, above the trigger (W3C CSS Anchor Positioning, position-try-fallbacks: flip-block, https://www.w3.org/TR/css-anchor-position-1/#fallback-var)", () => {
  async function assertFlippedAbove(page: Page, triggerSelector: string, contentSelector: string) {
    const trigger = await rectOf(page, triggerSelector);
    const content = await rectOf(page, contentSelector);
    const viewport = await viewportSize(page);
    const debug = JSON.stringify({ trigger, content, viewport });
    // Fully inside the viewport -- not clipped or partially off the bottom
    // edge, which is what the *preferred* (unflipped) placement would do
    // here (the trigger sits only a few px from the bottom edge).
    expect(content.top, debug).toBeGreaterThanOrEqual(-EDGE_TOLERANCE);
    expect(content.bottom, debug).toBeLessThanOrEqual(viewport.height + EDGE_TOLERANCE);
    // And specifically *above* the trigger, not merely inside the
    // viewport by some other accident (e.g. shift/clamp) -- a real flip.
    expect(content.bottom, debug).toBeLessThanOrEqual(trigger.top + EDGE_TOLERANCE);
  }

  test("CALIBRATION: native <div popover> + position-try-fallbacks: flip-block flips above", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#edge-bottom-native-trigger").click();
    await expect(page.locator("#edge-bottom-native-content")).toBeVisible();
    await assertFlippedAbove(page, "#edge-bottom-native-trigger", "#edge-bottom-native-content");
  });

  test("Tooltip content flips above its bottom-edge trigger", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#edge-bottom-tooltip-trigger").hover();
    await expect(page.locator("#edge-bottom-tooltip-content")).toBeVisible();
    await assertFlippedAbove(page, "#edge-bottom-tooltip-trigger", "#edge-bottom-tooltip-content");
  });

  test("HoverCard content flips above its bottom-edge trigger", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#edge-bottom-hovercard-trigger").hover();
    await expect(page.locator("#edge-bottom-hovercard-content")).toBeVisible();
    await assertFlippedAbove(page, "#edge-bottom-hovercard-trigger", "#edge-bottom-hovercard-content");
  });

  test("Popover (non-modal) content flips above its bottom-edge trigger", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#edge-bottom-popover-trigger").evaluate((el) => (el as HTMLElement).click());
    await expect(page.locator("#edge-bottom-popover-content")).toBeVisible();
    await assertFlippedAbove(page, "#edge-bottom-popover-trigger", "#edge-bottom-popover-content");
  });

  // Why this case exists: a Navbar cut off at the bottom/top must flip like
  // the other menus (a real page footer needs it).
  // Unlike the three cases above, `NavbarContent` has no `side`
  // prop (always below/start-aligned, the same fixed convention
  // `MenubarContent`/`DropdownMenuContent` use) -- so this pins the
  // *trigger* at the bottom edge instead, the shape a real page footer or
  // bottom nav bar puts a `Navbar` in. A plain `position: absolute;
  // top: 100%`-only div never flips: its content renders straight off the
  // bottom of the viewport, uncorrected.
  test("Navbar content flips above its bottom-edge trigger", async ({ page }) => {
    await gotoFixture(page);
    // `.hover()`, not `.click()` -- see the identical comment on this
    // fixture's Rule 1 "Navbar content escapes the clip" case above.
    await page.locator("#edge-bottom-navbar-trigger").hover();
    await expect(page.locator("#edge-bottom-navbar-content")).toBeVisible();
    await assertFlippedAbove(page, "#edge-bottom-navbar-trigger", "#edge-bottom-navbar-content");
  });

  // dev-docs/backlog.md row 72. Like Navbar just above,
  // `NavigationMenuContent` has no `side` prop (always below/start-aligned
  // -- see `primitives/src/navigation_menu.rs`'s
  // `NavigationMenuContentRendered`, hardcoded `ContentSide::Bottom`/
  // `ContentAlign::Start`), so this pins the *trigger* at the bottom edge
  // instead, the same construction as the Navbar case.
  test("NavigationMenu content flips above its bottom-edge trigger", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#edge-bottom-navigation-menu-trigger").hover();
    await expect(page.locator("#edge-bottom-navigation-menu-content")).toBeVisible();
    await assertFlippedAbove(page, "#edge-bottom-navigation-menu-trigger", "#edge-bottom-navigation-menu-content");
  });
});

test.describe("Rule 6 — inline-axis flip: a right-edge trigger with side=\"right\" renders its content fully inside the viewport, to the left of the trigger (W3C CSS Anchor Positioning, position-try-fallbacks: flip-inline, https://www.w3.org/TR/css-anchor-position-1/#fallback-var)", () => {
  async function assertFlippedLeft(page: Page, triggerSelector: string, contentSelector: string) {
    const trigger = await rectOf(page, triggerSelector);
    const content = await rectOf(page, contentSelector);
    const viewport = await viewportSize(page);
    const debug = JSON.stringify({ trigger, content, viewport });
    expect(content.left, debug).toBeGreaterThanOrEqual(-EDGE_TOLERANCE);
    expect(content.right, debug).toBeLessThanOrEqual(viewport.width + EDGE_TOLERANCE);
    expect(content.right, debug).toBeLessThanOrEqual(trigger.left + EDGE_TOLERANCE);
  }

  // All three anchor-positioned components (Tooltip, HoverCard, non-modal
  // Popover) accept every `ContentSide` value, so the inline axis applies
  // to all three -- no "block-axis only" narrowing needed here.
  test("CALIBRATION: native <div popover> + position-try-fallbacks: flip-inline flips left", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#edge-right-native-trigger").click();
    await expect(page.locator("#edge-right-native-content")).toBeVisible();
    await assertFlippedLeft(page, "#edge-right-native-trigger", "#edge-right-native-content");
  });

  test("Tooltip content flips left of its right-edge trigger", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#edge-right-tooltip-trigger").hover();
    await expect(page.locator("#edge-right-tooltip-content")).toBeVisible();
    await assertFlippedLeft(page, "#edge-right-tooltip-trigger", "#edge-right-tooltip-content");
  });

  test("HoverCard content flips left of its right-edge trigger", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#edge-right-hovercard-trigger").hover();
    await expect(page.locator("#edge-right-hovercard-content")).toBeVisible();
    await assertFlippedLeft(page, "#edge-right-hovercard-trigger", "#edge-right-hovercard-content");
  });

  test("Popover (non-modal) content flips left of its right-edge trigger", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#edge-right-popover-trigger").evaluate((el) => (el as HTMLElement).click());
    await expect(page.locator("#edge-right-popover-content")).toBeVisible();
    await assertFlippedLeft(page, "#edge-right-popover-trigger", "#edge-right-popover-content");
  });
});

test.describe("Rule 7 — normal mid-viewport placement does not flip", () => {
  // Regression guard for the primitives/src/top_layer.rs
  // `use_anchor_position_fallback` interaction: that hook's JS fallback
  // accepts *either* the primary or the flipped placement as
  // contract-legal (see its doc) precisely so it never fights a real CSS
  // flip -- this rule confirms the common (nothing-needs-to-flip) case
  // still renders at the primary, unflipped placement, i.e. that the
  // acceptance does not widen to "accept anything." Scrolled into
  // view before opening (see Rule 4's comment above) -- this rule's whole
  // premise is "plenty of room," which the fixture's below-the-fold
  // resting position (the stacking section is in-flow) does not give it.
  test("Popover (non-modal) with plenty of room stays on its preferred (bottom) side", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#stack-sibling").scrollIntoViewIfNeeded();
    await page.locator("#stack-popover-trigger").evaluate((el) => (el as HTMLElement).click());
    await expect(page.locator("#stack-popover-content")).toBeVisible();
    const trigger = await rectOf(page, "#stack-popover-trigger");
    const content = await rectOf(page, "#stack-popover-content");
    const debug = JSON.stringify({ trigger, content });
    // Below the trigger (side="bottom", unflipped) -- a block-axis flip
    // would put it above instead.
    expect(content.top, debug).toBeGreaterThanOrEqual(trigger.bottom - EDGE_TOLERANCE);
  });
});

/**
 * Rule 8 — scroll tracking (a `ColorPicker` popup must not float
 * viewport-fixed while the page scrolls, detached from its trigger). Two independent positioning paths
 * exist (`primitives/src/top_layer.rs`'s `use_anchor_position_fallback`
 * doc, "Scroll/resize tracking"), so both need their own case here:
 *
 *   - The CSS-anchor path (native `anchor()` in the stylesheet): the
 *     platform re-resolves this on scroll for free, with no JS involved at
 *     all -- this fixture's own `Popover`/`Tooltip`/`HoverCard` all land on
 *     this path in this repo's Chromium once `popover/style.css`'s
 *     `@supports` block is loaded, which is the case for every page in this
 *     workspace *except* `color_picker`'s own unless its sheet wires it in
 *     (see `preview/src/components/color_picker/style.css`) -- so the
 *     fixture page used here for the CSS-path case is `color_picker`, the
 *     composition the scroll-tracking rule is about, not this
 *     file's own `TopLayerFixture`.
 *   - The JS-measured fallback path (`use_anchor_position_fallback`'s
 *     "neither matches" branch): only engages when the CSS path failed to
 *     land the content correctly. Whether an engine's CSS path lands this
 *     fixture's stacking `Popover` is the engine's business (Chromium 153
 *     does, backlog row 378), so the case forces the fallback itself:
 *     `stripAnchorSupportsBlock` removes the `anchor()` rules before the
 *     open, the no-anchor engine Rule 11(c) simulates, and the inline
 *     `top`/`left` check below still proves the fallback is what ran.
 *
 * Both cases assert the same thing: the content-to-trigger offset,
 * measured as content's top-left corner relative to the trigger's
 * bounding box, is unchanged after a 100-150px scroll. `toBeCloseTo(_, 0)`
 * (whole-pixel tolerance) accounts for subpixel rounding across the two
 * `getBoundingClientRect()` reads, not for any real drift.
 */
test.describe("Rule 8 — scroll tracking: an anchored overlay's content keeps its position relative to its trigger while the page scrolls", () => {
  test("Popover (non-modal) on the JS-measured fallback path tracks its trigger through a scroll", async ({ page }) => {
    await gotoFixture(page);
    // Scrolled comfortably into view *before* opening (same reasoning as
    // Rule 4/7 above): this rule's own 100-150px scroll happens *after*
    // opening, deliberately -- that is the thing under test -- so the
    // starting position needs enough headroom on both axes that the modest
    // follow-up scroll below cannot itself cross a flip threshold (which
    // would legitimately change the offset for a reason unrelated to
    // tracking, and this rule isn't the one guarding flip behavior --
    // Rules 5-7 already do).
    await page.locator("#stack-sibling").scrollIntoViewIfNeeded();
    // The JS path by construction, on any engine: no `anchor()` rules, so
    // the CSS placement cannot match and the fallback takes over at open.
    await stripAnchorSupportsBlock(page);
    await page.locator("#stack-popover-trigger").evaluate((el) => (el as HTMLElement).click());
    await expect(page.locator("#stack-popover-content")).toBeVisible();

    // Confirm this really is the fallback path (the case this rule exists
    // to cover): the fallback is the only thing that ever writes an
    // inline `top` on this element (see `use_anchor_position_fallback`).
    const inlineTop = await page.locator("#stack-popover-content").evaluate(
      (el) => (el as HTMLElement).style.top,
    );
    expect(inlineTop, "expected the JS fallback path to be active here").not.toBe("");

    const offsetOf = async () =>
      page.evaluate(() => {
        // rest-ok: the offset between two boxes read in one task after the open settled; the test is its stability under scrolling
        const c = document.getElementById("stack-popover-content")!.getBoundingClientRect();
        const t = document.getElementById("stack-popover-trigger")!.getBoundingClientRect(); // rest-ok: same task, same reason
        return { top: c.top - t.bottom, left: c.left - t.left };
      });

    const before = await offsetOf();
    await page.evaluate(() => window.scrollBy(0, 120));
    // The fix is rAF-throttled (`use_anchor_position_fallback`'s
    // `rafScheduled` flag) -- a couple of frames is enough for it to land.
    await page.waitForTimeout(150);
    const after = await offsetOf();

    const debug = JSON.stringify({ before, after });
    expect(after.top, debug).toBeCloseTo(before.top, 0);
    expect(after.left, debug).toBeCloseTo(before.left, 0);
  });

  test("ColorPicker popover on the CSS-anchor path tracks its trigger through a scroll", async ({ page }) => {
    // A viewport with room for the whole panel below the trigger, so the 150px scroll below cannot cross the flip
    // threshold (see the "headroom" note in the case above). The panel is 386px tall and opens 8px under a trigger
    // whose bottom edge is at y ~ 453, so at Playwright's default 1280x720 it does NOT fit: `position-try-fallbacks:
    // flip-block` correctly opens it ABOVE the trigger (offset -428), and the scroll then frees enough room below
    // that it flips back (offset +8) -- a 436px jump that reads as "frozen / drifted" but is the flip, not a tracking
    // failure. On a release SSG build: viewport 720 gives -428 then 8; 900/1000/1100 give 8 then 8, exactly
    // tracked on the CSS-anchor path (empty inline `top`). dev-docs/backlog.md rows 21/106.
    await page.setViewportSize({ width: 1280, height: 1000 });
    // dev-docs/backlog.md row 109: SSG lane -- `gotoHydrated`, see this
    // file's own `gotoFixture` doc.
    await gotoHydrated(page, `${BASE_URL}/component/color_picker/`, {
      timeout: NAV_TIMEOUT,
    });

    // Opened at scrollY=0 with a raw DOM click (bypassing Playwright's own
    // actionability auto-scroll), *not* pre-scrolled: this repo's Chromium
    // build computes a `[popover]` + `position: fixed` element's *very first*
    // `anchor()`-resolved position incorrectly whenever the document is
    // already scrolled at the moment the popover is shown -- off by
    // exactly the scroll offset, as if measured document-relative instead
    // of viewport-relative -- and that wrong value never self-corrects for
    // the life of that popover instance, even once real further scrolling
    // happens. `use_anchor_position_fallback`'s own fallback detects and
    // compensates for this (see its doc), so
    // end-user positioning is never wrong; this test's whole point, though,
    // is to isolate the *CSS-only* path specifically, which this Chromium
    // quirk can only be kept out of by not scrolling before the open it is
    // asserting about. Room below the trigger DOES matter at the default
    // 720px-tall viewport (the panel flips above it there -- see the
    // viewport note at the top of this case), which is why the viewport is
    // enlarged above rather than the flip being ignored.
    const trigger = demoRole(page, "button");
    await expect(trigger).toBeVisible();
    await trigger.evaluate((el) => (el as HTMLElement).click());
    const content = page.getByRole("dialog");
    await expect(content).toBeVisible();

    // Confirm this really is the CSS-anchor path (no inline `top` -- the
    // fallback never engaged), so this test cannot pass merely because the
    // JS fallback's own tracking (already covered above) papered over a
    // CSS-anchor regression here.
    const inlineTop = await content.evaluate((el) => (el as HTMLElement).style.top);
    expect(inlineTop, "expected the CSS-anchor path to be active here").toBe("");

    // Let the open-state fade-in settle before taking the "before" reading:
    // `dx-color-picker-popover-fade-in` (`.15s ease-out`,
    // `color_picker/style.css`) transiently animates this element, and
    // `getBoundingClientRect()` mid-animation can
    // read a not-yet-settled box on this path -- unrelated to tracking (it
    // resolves to the same final rect either way), but this rule needs a
    // stable "before" to compare "after" against.
    await page.waitForTimeout(200);

    // This picker's own trigger and panel -- NOT `document.querySelector('[style*="anchor-name"]')` /
    // `("dialog[popover]")`: the header's theme picker (a `.dx-theme-picker-trigger` button with an inline
    // `anchor-name`, and its own popover dialog) precedes the demo in the DOM, so those first-match selectors
    // would measure the header, which does not scroll with the content.
    const offsetOf = async () => {
      const t = await readAtRest(trigger, (el) => el.getBoundingClientRect().toJSON());
      const c = await readAtRest(content, (el) => el.getBoundingClientRect().toJSON());
      return { top: c.top - t.bottom, left: c.left - t.left };
    };

    const before = await offsetOf();
    await page.evaluate(() => window.scrollBy(0, 150));
    await page.waitForTimeout(150);
    const after = await offsetOf();

    const debug = JSON.stringify({ before, after });
    expect(after.top, debug).toBeCloseTo(before.top, 0);
    expect(after.left, debug).toBeCloseTo(before.left, 0);
  });
});

/**
 * Rule 9 — point-positioned vs. anchored scroll behavior. Rule 8 above covers the *anchored* case (content tracks its
 * trigger through a scroll); this rule adds the two shapes `ContextMenu` and
 * `Menubar` take:
 *
 *   - `ContextMenu` opens at a raw click point with **no** anchor at all
 *     (`ContextMenuContentRendered`, `context_menu.rs` -- deliberately not
 *     anchored: CSS anchors need an element). A `position: fixed` div's
 *     containing block is the viewport (absent a transformed ancestor), so
 *     the content stays at the click's *viewport-relative* position and
 *     does not move on screen as the page scrolls underneath it -- the same
 *     thing a native OS context menu does, and what Radix's own
 *     `ContextMenu` does. Promoting it to the top layer via
 *     `popover="manual"` does not change this: a top-layer element's
 *     containing block is *also* the viewport, so the inline `top`/`left`
 *     pixel values set at open time keep meaning exactly what they mean for
 *     a `position: fixed` div.
 *   - `Menubar` menus, by contrast, anchor each menu's content to its own
 *     trigger (`MenubarContentRendered`, `menubar.rs`) -- the same shape
 *     `DropdownMenu`'s Rule 8 case exercises -- and so must keep tracking
 *     that trigger through a scroll exactly as Rule 8 already checks for
 *     `Popover`.
 *
 * Fixture: the "Point-positioned vs. anchored scroll behavior" section of
 * `TopLayerFixture` -- both controls sit normally in-flow, away from any
 * clip ancestor or viewport edge, so a 120px scroll cannot itself cross a
 * flip threshold (Rules 5-7 already guard flipping; this rule is only about
 * what happens to already-open content during a scroll).
 */
test.describe("Rule 9 — point-positioned vs. anchored scroll behavior", () => {
  test("ContextMenu content stays at its click's viewport position while the page scrolls (does not track the page)", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#scroll-context-menu-trigger").scrollIntoViewIfNeeded();
    await page.locator("#scroll-context-menu-trigger").click({ button: "right" });
    await expect(page.locator("#scroll-context-menu-content")).toBeVisible();

    const rectOfContent = () => rectOf(page, "#scroll-context-menu-content");
    const before = await rectOfContent();
    await page.evaluate(() => window.scrollBy(0, 120));
    // No positioning logic runs on scroll for this component at all (no
    // fallback, no listener) -- a short settle is still given so this
    // assertion cannot pass merely because it ran before a paint.
    await page.waitForTimeout(150);
    const after = await rectOfContent();

    const debug = JSON.stringify({ before, after });
    // Viewport-relative position unchanged -- the content did not move on
    // screen, i.e. it did *not* track the page's scroll.
    expect(after.top, debug).toBeCloseTo(before.top, 0);
    expect(after.left, debug).toBeCloseTo(before.left, 0);
  });

  // Row 193: a menubar never locks the page, so an open menu does not follow its trigger through a
  // scroll -- any scroll (past the helper's 300 ms quiet window after the open) closes it
  // (`top_layer::use_close_on_scroll`, `playwright/close-on-scroll.spec.ts` for the full policy).
  test("Menubar content closes on a scroll instead of following its trigger", async ({ page }) => {
    await gotoFixture(page);
    await page.locator("#scroll-menubar-trigger").scrollIntoViewIfNeeded();
    await page.locator("#scroll-menubar-trigger").click();
    await expect(page.locator("#scroll-menubar-content")).toBeVisible();
    await page.waitForTimeout(450); // past the quiet window that follows an open

    await page.evaluate(() => window.scrollBy(0, 120));
    await expect(page.locator("#scroll-menubar-content")).toHaveCount(0);
  });
});



/**
 * Rule 11 — anchored-overlay self-overlap contract, iOS on-screen keyboard
 * (the Combobox options list must not render on top of its own search
 * input after focusing/typing on iOS Safari).
 *
 * Citation: no single WHATWG/W3C section states "an anchored popup must
 * never cover its own anchor" as a normative MUST -- this rule cites this
 * repo's own anchored-overlay contract (the `[data-side]`/`[data-align]`
 * placement formula `use_anchor_position_fallback` implements and
 * self-checks, `primitives/src/top_layer.rs`) plus the platform surface the
 * contract is built on:
 *   - CSSOM View's `VisualViewport` -- the on-screen-keyboard-aware
 *     viewport rectangle. `window.innerWidth`/`innerHeight` do NOT shrink
 *     when iOS Safari's keyboard appears; `window.visualViewport` does:
 *     <https://www.w3.org/TR/cssom-view-1/#dom-window-visualviewport>
 *     <https://www.w3.org/TR/cssom-view-1/#the-visualviewport-interface>
 *   - The WHATWG HTML popover UA stylesheet default this file's Rule 1-10
 *     already cite -- what an anchored overlay falls back to once CSS
 *     Anchor Positioning cannot place it:
 *     <https://html.spec.whatwg.org/multipage/rendering.html#the-popover-attribute>
 *
 * Two gaps `use_anchor_position_fallback` must not have:
 *   1. No visual-viewport awareness -- a fallback that reads only
 *      `window.innerWidth/innerHeight` + `getBoundingClientRect()` is
 *      blind to a keyboard-driven `visualViewport` change.
 *   2. Scroll/resize tracking listeners installed ONLY when the
 *      *first* measurement already concluded `usingFallback` (see that
 *      hook's own "Scroll/resize tracking" doc) -- an overlay that looked
 *      correctly anchored at open time would never get a listener at all, so a
 *      later divergence (the keyboard opening after the overlay is already
 *      open -- exactly Combobox's shape: focusing its input both opens the
 *      listbox and summons the keyboard) would never be re-measured for the
 *      rest of that open.
 *
 * Scope: every overlay that actually calls `use_anchor_position_fallback`
 * (in `primitives/src/*.rs`, `combobox/components/list.rs`,
 * `select/components/list.rs`) -- Tooltip, HoverCard, Popover,
 * DropdownMenu, Menubar, Select, Combobox. `ContextMenu`'s root content is deliberately
 * excluded: per Rule 9's own doc above and `context_menu.rs` (its root content is
 * clamped by `use_point_anchor_clamp`, not `use_anchor_position_fallback`), it opens at
 * a raw click point with no anchor at all, so it has no "own anchor rect"
 * to overlap and neither of the two gaps can apply to it.
 *
 * Parts (a)-(c) below use the "Clipping escape" section's
 * `clip-*-trigger`/`clip-*-content` pairs (one per component, already used
 * by Rule 1 above) -- reused rather than duplicated, since they already
 * give every one of these seven components its own persistent,
 * id-addressable trigger and content.
 *
 * Simulating an iOS Safari engine in this sandbox's Chromium:
 *   - No Chromium launch flag disables `anchor-name`/`anchor()` support
 *     (the feature has fully shipped and is not flag-gated) -- so "this
 *     engine doesn't do CSS Anchor Positioning" is
 *     simulated at the CSS-cascade level instead, in
 *     `stripAnchorSupportsBlock` below: brace-counted removal (a greedy
 *     regex would over/under-match the many nested `{}` inside it) of the
 *     `@supports (anchor-name: --a) { ... }` block's body -- the only
 *     top-level `@supports` in the engine-injected
 *     `#dx-anchor-positioning-styles` stylesheet
 *     (`primitives/src/top_layer.rs`'s `anchor_positioning_inject_js`).
 *     This drops exactly the `anchor()`-based placement rules an engine
 *     without the feature would never have applied, regardless of whether
 *     *this* browser actually supports it.
 *   - A real iOS keyboard shrinks `window.visualViewport` while leaving
 *     `window.innerWidth/innerHeight` untouched; `page.setViewportSize()`
 *     cannot reproduce that split -- it resizes the real layout viewport
 *     (`window.inner*` included). What it, plus a `mouse.wheel` scroll,
 *     *can* exercise is gap 2 above: does anything re-measure at all once
 *     an overlay is already open? That is what parts (b) and (c) below
 *     drive at. (Real `visualViewport`-only divergence needs a genuine iOS
 *     Safari; tracked in `dev-docs/backlog.md`.)
 *
 * Part (c)'s second sub-case, "conforming at open, diverging later," is the
 * one gap 2 exists for and needs a component that genuinely lands on the
 * CSS-anchor path at open, not the JS fallback: every one of the seven
 * `clip-*` pairs above runs
 * the JS fallback at open in this fixture context (the `clip-box`
 * ancestor's `transform` appears to invalidate `anchor()`'s reference the
 * same way it does for `use_anchor_position_fallback`'s own cases), and
 * DropdownMenu/Select/Combobox land on the fallback path even
 * on their own, unclipped gallery pages -- the same broad CSS-Anchor-
 * Positioning unevenness `dev-docs/backlog.md` item 21 documents for
 * ColorPicker/DatePicker in this sandbox. `CONFORMING_CASES` below is the
 * (smaller) set of contexts on this same fixture, or a
 * component's own gallery page, that DO land on the CSS-anchor path at
 * open in this sandbox; the rest are explicitly marked unsupported here
 * with the reason, rather than asserting something that is not true.
 */
function rectsIntersect(
  a: { top: number; left: number; right: number; bottom: number },
  b: { top: number; left: number; right: number; bottom: number },
): boolean {
  return a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;
}

async function rectOfLocator(locator: import("@playwright/test").Locator) {
  return readAtRest(locator, (el) => {
    const r = (el as HTMLElement).getBoundingClientRect();
    return { top: r.top, left: r.left, right: r.right, bottom: r.bottom, width: r.width, height: r.height };
  });
}

/** Roughly an iPhone viewport -- see this section's header doc. */
const MOBILE_VIEWPORT = { width: 390, height: 844 };
/**
 * Roughly the visible height left on that viewport once an iOS Safari
 * on-screen keyboard (QWERTY + accessory bar, ~45% of viewport height) is
 * showing. Only the height changes -- a keyboard narrows nothing.
 */
const KEYBOARD_VIEWPORT = { width: 390, height: 460 };

/**
 * Pins `locator` a fixed 24px from the top of the viewport, rather than
 * `scrollIntoViewIfNeeded`'s browser-chosen alignment: parts (b)/(c) below
 * shrink the viewport's *height* after opening (simulating the keyboard),
 * and a trigger left near the bottom of a tall starting viewport could
 * scroll out of the shrunk one for a reason that has nothing to do with
 * this rule (not enough headroom left in the fixture's own layout) --
 * pinning it near the top guarantees the trigger (and, by the placement
 * contract, its content) has room to stay on-screen in
 * `KEYBOARD_VIEWPORT` regardless of where this section falls in page flow.
 */
async function pinNearTop(page: Page, locator: import("@playwright/test").Locator): Promise<void> {
  // A viewport-pinned trigger cannot be scrolled anywhere, so "pin it near the
  // top" is meaningless for one -- and worse than meaningless: `rect.top` for a
  // `position: fixed` element is always its fixed on-screen position, so the
  // scroll below computes a large, arbitrary offset (+659px for
  // `#edge-bottom-popover-trigger`, whose row is `position: fixed; bottom: 4px`)
  // that scrolls the *rest* of the page under the stationary trigger. That drags
  // the fixture's unrelated `#stack-sibling` (`position: absolute; z-index: 9999`,
  // belonging to the light-dismiss/stacking section) into the same on-screen
  // rectangle, where it intercepts the click and sends Playwright into a
  // scroll-and-retry loop until the 5-minute test timeout.
  //
  // The guard lives here rather than in Rule 11(c)'s Popover case because the
  // hazard belongs to this helper's own assumption (that a trigger sits in normal
  // document flow), so any fixed-position trigger would hit it.
  const isViewportPinned = await locator.evaluate((el) => {
    for (let node: Element | null = el; node; node = node.parentElement) {
      if (getComputedStyle(node).position === "fixed") return true;
    }
    return false;
  });
  if (isViewportPinned) return;

  // Centre the trigger horizontally inside its own scroll container first:
  // on the 390px mobile viewport the fixture's clip box is wider than the
  // page, and a trigger hanging past the right edge collides with the
  // fixture's fixed right-edge triggers, which then intercept the click and
  // send Playwright into its scroll-and-retry loop.
  //
  // `pinNearTop`'s entire contract (see this function's own opening doc) is
  // to deliver the trigger at a settled, known position BEFORE
  // `overlay.open()` runs. The root `scroll-behavior` is `auto` unless a
  // same-page anchor jump opts in (`html[data-anchor-scroll]`; dev-docs/
  // backlog.md rows 110 and 228), so an unqualified scroll -- one that does
  // not name its own `behavior` -- is already instant without a per-call-site
  // argument. Kept unqualified (no `behavior: "instant"` here) so this helper
  // relies on the page's own contract, not a copy of a workaround.
  await locator.evaluate((el) => el.scrollIntoView({ block: "nearest", inline: "center" }));
  const rect = await rectOfLocator(locator);
  // "Near the top" means just below the sticky navbar, not 24px from the
  // viewport edge: the preview's `.dx-preview-navbar` is `position: sticky;
  // top: 0` (main.css), so anything pinned above its bottom edge is covered
  // and hit-tested by the nav -- Playwright then retries `hover()` until the
  // test times out (this needs main.css applied; see
  // `oracle/tier2-html/global-stylesheet.spec.ts`).
  const navBottom = await page.evaluate(() => {
    const nav = document.querySelector(".dx-preview-navbar, nav[aria-label='Primary']");
    return nav ? nav.getBoundingClientRect().bottom : 0; // rest-ok: the page's fixed navbar, not an overlay
  });
  await page.evaluate(
    ([top, offset]) => window.scrollTo({ top: window.scrollY + top - offset, left: 0 }),
    [rect.top, navBottom + 24] as const,
  );
}

/**
 * See this section's header doc, "Simulating an iOS Safari engine" -- the
 * `page.setViewportSize` + `mouse.wheel` half of the simulation (gap 2:
 * does anything re-measure once already open).
 */
async function simulateOnScreenKeyboard(page: Page): Promise<void> {
  await page.setViewportSize(KEYBOARD_VIEWPORT);
  await page.mouse.wheel(0, 40);
  // rAF-throttled (`use_anchor_position_fallback`'s `rafScheduled` flag,
  // plus its settle loop) -- give tracking a few frames to land.
  await page.waitForTimeout(250);
}

/**
 * See this section's header doc, "Simulating an iOS Safari engine" -- the
 * CSS-cascade half (`stripAnchorSupportsBlock`).
 */
async function stripAnchorSupportsBlock(page: Page): Promise<void> {
  await page.waitForFunction(() => !!document.getElementById("dx-anchor-positioning-styles"), {
    timeout: 15000,
  });
  const result = await page.evaluate(() => {
    const tag = document.getElementById("dx-anchor-positioning-styles") as HTMLStyleElement;
    const src = tag.textContent ?? "";
    const startMarker = "@supports (anchor-name: --a)";
    const idx = src.indexOf(startMarker);
    if (idx === -1) return { ok: false, reason: "marker not found" };
    let i = src.indexOf("{", idx);
    let depth = 0;
    let end = -1;
    for (; i < src.length; i++) {
      if (src[i] === "{") depth++;
      else if (src[i] === "}") {
        depth--;
        if (depth === 0) {
          end = i + 1;
          break;
        }
      }
    }
    if (end === -1) return { ok: false, reason: "unbalanced braces" };
    tag.textContent = src.slice(0, idx) + src.slice(end);
    return { ok: true };
  });
  if (!result.ok) {
    throw new Error(`stripAnchorSupportsBlock failed: ${JSON.stringify(result)}`);
  }
}

interface AnchoredOverlay {
  name: string;
  triggerId: string;
  contentId: string;
  open: (page: Page) => Promise<void>;
  /**
   * Hover-driven overlays (Tooltip/HoverCard) can lose `:hover` when the
   * viewport shrinks and a `mouse.wheel` scroll moves the trigger out from
   * under the (stationary) synthetic pointer -- irrelevant to what this
   * rule checks (placement, not open/close persistence), so those two
   * re-assert the hover after the keyboard-simulation step. A no-op for
   * every click/keyboard-driven overlay.
   */
  reengage?: (page: Page) => Promise<void>;
}

const ANCHORED_OVERLAYS: AnchoredOverlay[] = [
  {
    name: "Tooltip",
    triggerId: "clip-tooltip-trigger",
    contentId: "clip-tooltip-content",
    open: async (page) => {
      await page.locator("#clip-tooltip-trigger").hover();
    },
    reengage: async (page) => {
      await page.locator("#clip-tooltip-trigger").hover();
    },
  },
  {
    name: "HoverCard",
    triggerId: "clip-hovercard-trigger",
    contentId: "clip-hovercard-content",
    open: async (page) => {
      await page.locator("#clip-hovercard-trigger").hover();
    },
    reengage: async (page) => {
      await page.locator("#clip-hovercard-trigger").hover();
    },
  },
  {
    name: "Popover",
    triggerId: "clip-popover-trigger",
    contentId: "clip-popover-content",
    open: async (page) => {
      await page.locator("#clip-popover-trigger").click();
    },
  },
  {
    name: "DropdownMenu",
    triggerId: "clip-dropdown-menu-trigger",
    contentId: "clip-dropdown-menu-content",
    open: async (page) => {
      await page.locator("#clip-dropdown-menu-trigger").click();
    },
  },
  {
    name: "Menubar",
    triggerId: "clip-menubar-trigger",
    contentId: "clip-menubar-content",
    open: async (page) => {
      await page.locator("#clip-menubar-trigger").click();
    },
  },
  {
    name: "Select",
    triggerId: "clip-select-trigger",
    contentId: "clip-select-content",
    open: async (page) => {
      await page.locator("#clip-select-trigger").click();
    },
  },
  {
    name: "Combobox",
    triggerId: "clip-combobox-trigger",
    contentId: "clip-combobox-content",
    // A click alone does not open this popup (Rule 1's identical case
    // above) -- ArrowDown does.
    open: async (page) => {
      await page.locator("#clip-combobox-trigger").click();
      await page.keyboard.press("ArrowDown");
    },
  },
  {
    // `use_anchor_position_fallback` is a shared function, so the same iOS
    // keyboard/self-overlap contract Rule 11 checks for every other
    // `dx-anchor-*` consumer must hold for Navbar too, not just the
    // flip/clip rules.
    // `.hover()`, not `.click()` -- see the identical comment on this
    // fixture's Rule 1 "Navbar content escapes the clip" case; `reengage`
    // is needed for the same reason Tooltip/HoverCard above need it (a
    // real, hover-driven overlay, not click/keyboard-driven).
    name: "Navbar",
    triggerId: "clip-navbar-trigger",
    contentId: "clip-navbar-content",
    open: async (page) => {
      await page.locator("#clip-navbar-trigger").hover();
    },
    reengage: async (page) => {
      await page.locator("#clip-navbar-trigger").hover();
    },
  },
  {
    // dev-docs/backlog.md row 72: `NavigationMenuContent` also calls
    // `use_anchor_position_fallback` (see this module's own "Top layer"
    // doc), so the same iOS keyboard/self-overlap contract applies here
    // too, mirroring the Navbar entry above exactly (`.hover()`, not
    // `.click()`, plus `reengage` for the same hover-driven reason).
    name: "NavigationMenu",
    triggerId: "clip-navigation-menu-trigger",
    contentId: "clip-navigation-menu-content",
    open: async (page) => {
      await page.locator("#clip-navigation-menu-trigger").hover();
    },
    reengage: async (page) => {
      await page.locator("#clip-navigation-menu-trigger").hover();
    },
  },
];

async function assertNoSelfOverlapWithinViewport(page: Page, overlay: AnchoredOverlay) {
  const trigger = await rectOf(page, `#${overlay.triggerId}`);
  const content = await rectOf(page, `#${overlay.contentId}`);
  const viewport = await viewportSize(page);
  const debug = JSON.stringify({ overlay: overlay.name, trigger, content, viewport });
  expect(rectsIntersect(content, trigger), debug).toBe(false);
  expect(content.top, debug).toBeGreaterThanOrEqual(-EDGE_TOLERANCE);
  expect(content.bottom, debug).toBeLessThanOrEqual(viewport.height + EDGE_TOLERANCE);
}

test.describe("Rule 11 — anchored-overlay self-overlap contract", () => {
  test.describe("(a) no self-overlap once open", () => {
    for (const overlay of ANCHORED_OVERLAYS) {
      test(`${overlay.name}: content never overlaps its own anchor`, async ({ page }) => {
        await gotoFixture(page);
        await pinNearTop(page, page.locator(`#${overlay.triggerId}`));
        await overlay.open(page);
        await expect(page.locator(`#${overlay.contentId}`)).toBeVisible();
        await assertNoSelfOverlapWithinViewport(page, overlay);
      });
    }
  });

  test.describe("(b) simulated iOS on-screen keyboard (viewport shrink + scroll, after open)", () => {
    for (const overlay of ANCHORED_OVERLAYS) {
      test(`${overlay.name}: re-measures and stays clear of its anchor once the keyboard opens`, async ({
        page,
      }) => {
        await page.setViewportSize(MOBILE_VIEWPORT);
        await gotoFixture(page);
        await pinNearTop(page, page.locator(`#${overlay.triggerId}`));
        await overlay.open(page);
        await expect(page.locator(`#${overlay.contentId}`)).toBeVisible();

        await simulateOnScreenKeyboard(page);
        await overlay.reengage?.(page);

        await assertNoSelfOverlapWithinViewport(page, overlay);
      });
    }
  });

  test.describe("(c) simulated no-anchor engine (CSS Anchor Positioning support stripped)", () => {
    for (const overlay of ANCHORED_OVERLAYS) {
      test(`${overlay.name}: no anchor support from the start, then the keyboard opens`, async ({ page }) => {
        await page.setViewportSize(MOBILE_VIEWPORT);
        await gotoFixture(page);
        await stripAnchorSupportsBlock(page);
        await pinNearTop(page, page.locator(`#${overlay.triggerId}`));
        await overlay.open(page);
        await expect(page.locator(`#${overlay.contentId}`)).toBeVisible();

        await simulateOnScreenKeyboard(page);
        await overlay.reengage?.(page);

        await assertNoSelfOverlapWithinViewport(page, overlay);
      });
    }

    /**
     * The gap this whole rule exists for (gap 2 in this section's header
     * doc): CSS Anchor Positioning is fully intact and correct at *open*
     * time -- `usingFallback` stays false, so a listener attached only on the
     * fallback path would never exist -- and only THEN does anchor-positioning
     * support disappear, mid-open, the same moment an iOS keyboard would
     * appear over a focused Combobox input.
     *
     * Needs a context that genuinely lands on the CSS-anchor path at open
     * (see this section's header doc, last paragraph, for why the `clip-*`
     * pairs above cannot be reused for this one case) -- `CONFORMING_CASES`
     * below, checked per entry, most deliberately marked
     * unsupported with the reason rather than faked:
     *   - DropdownMenu/Select/Combobox never land on the CSS-anchor path at
     *     all in this sandbox, on any page checked (their own gallery page
     *     or this fixture) -- same broad unevenness `dev-docs/backlog.md` item
     *     21 documents.
     *   - Tooltip/HoverCard DO conform at open here (`edge-bottom-*`), but
     *     shrinking the viewport for the keyboard
     *     simulation moves the trigger out from under Chromium's actual
     *     (unmoved) cursor, and this browser re-hit-tests `:hover` on
     *     scroll/resize for real -- so the *real* browser closes the
     *     overlay via a genuine `pointerleave` before any of the fix's
     *     logic even runs, and re-hovering to keep it open starts a whole
     *     new open cycle (a fresh, cold measurement -- not the same-open
     *     "diverges mid-open" case this test exists for). Not a gap the
     *     fix closes or should try to: a hover-only surface does not
     *     summon an on-screen keyboard (Combobox's is a focused,
     *     keyboard-persistent text input).
     *   - Menubar conforms at open on its own gallery page at desktop
     *     width, but not at `MOBILE_VIEWPORT`
     *     width (390px) -- the one precondition this case needs.
     *   - Navbar (dev-docs/backlog.md row 41; part of `ANCHORED_OVERLAYS`
     *     above for parts (a)/(b), not of this `CONFORMING_CASES` table)
     *     opens the same way here as
     *     in `ANCHORED_OVERLAYS` above -- `.hover()` on its trigger, with
     *     a `reengage` step after the keyboard-simulation resize -- the
     *     exact shape that entry's own doc gives Tooltip/HoverCard
     *     `reengage` for (a hover-driven overlay that can lose `:hover`
     *     when the resize moves the trigger out from under the
     *     stationary synthetic cursor). That is the same Chromium
     *     hover/pointerleave behaviour the Tooltip/HoverCard bullet above
     *     describes -- a property of *how the
     *     overlay opens*, not of this component's own code -- so it
     *     applies here on the same grounds. Marked unsupported on that
     *     basis (a structural inference, not independently confirmed by
     *     execution for Navbar; dev-docs/backlog.md rows 36/41).
     *   - Popover is the sole survivor, and is enough: its content really
     *     does stop tracking after the strip (stays at
     *     `style.top === ""`, i.e. no re-measurement ever happens) --
     *     `expect(...).not.toBe("")` below is the direct, mechanism-level
     *     assertion for that (rather than an overlap/viewport check alone,
     *     which this exact fixture's geometry can pass by coincidence even
     *     while genuinely stale: the stale, UA-default position it falls
     *     back to for this specific trigger happens to land at the
     *     viewport's top-left corner, nowhere near -- but also nowhere
     *     *correct* relative to -- the trigger).
     */
    type ConformingCase =
      | {
          name: string;
          supported: true;
          goto: (page: Page) => Promise<void>;
          trigger: (page: Page) => import("@playwright/test").Locator;
          content: (page: Page) => import("@playwright/test").Locator;
          open: (page: Page) => Promise<void>;
        }
      | { name: string; supported: false; reason: string };

    const CONFORMING_CASES: ConformingCase[] = [
      {
        name: "Popover",
        supported: true,
        goto: gotoFixture,
        trigger: (page) => page.locator("#edge-bottom-popover-trigger"),
        content: (page) => page.locator("#edge-bottom-popover-content"),
        open: async (page) => {
          // Row 210 (exclusion zone): a CSS-anchor placement that leaves the visual viewport is not
          // trusted -- the JS path takes over at open. This fixture's trigger sits 16px from
          // the left edge, so its centre-aligned 288px panel would start 73px off-screen and
          // (correctly) take the JS path from the first frame, which is not the "conforming at
          // open" premise of this case. Moving the fixed trigger inward keeps the CSS placement
          // inside the viewport, so the CSS arm still places it and the divergence is the test.
          await page.locator("#edge-bottom-popover-trigger").evaluate((el) => {
            for (let node: HTMLElement | null = el as HTMLElement; node; node = node.parentElement) {
              if (getComputedStyle(node).position === "fixed") {
                node.style.left = "140px";
                node.style.right = "auto";
                break;
              }
            }
          });
          await page.locator("#edge-bottom-popover-trigger").click();
        },
      },
      {
        name: "Tooltip",
        supported: false,
        reason:
          "conforms at open (edge-bottom-tooltip-*) but is hover-driven: the keyboard-simulation resize " +
          "moves the trigger out from under the real (unmoved) cursor, and this Chromium genuinely closes " +
          "the tooltip via a real pointerleave before the tracking logic runs -- not the same-open " +
          "'diverges mid-open' case this test targets, and not the keyboard-overlap shape (a focused text " +
          "input, not a hover surface, is what keeps an overlay open while a keyboard appears).",
      },
      {
        name: "HoverCard",
        supported: false,
        reason: "same as Tooltip above -- hover-driven, genuinely closed by the keyboard-simulation resize.",
      },
      {
        name: "DropdownMenu",
        supported: false,
        reason:
          "no naturally CSS-anchor-conforming context found for DropdownMenu in this Chromium sandbox " +
          "(checked its own gallery page's demo instance and every clip-* pair on the top_layer fixture -- " +
          "all land on the JS fallback path at open already; dev-docs/backlog.md item 21 records this sandbox's " +
          "broader CSS-Anchor-Positioning unevenness). Verify on a real device instead.",
      },
      {
        name: "Menubar",
        supported: false,
        reason:
          "conforms at open on its own gallery page at desktop width, but confirmed by execution: not at " +
          "MOBILE_VIEWPORT width (390px) -- the fallback is already active before the strip step even runs, " +
          "so this specific case ('conforming at open') does not arise for it in this sandbox.",
      },
      {
        name: "Select",
        supported: false,
        reason:
          "no naturally CSS-anchor-conforming context found for Select in this Chromium sandbox (checked " +
          "its own gallery page's demo instance and every clip-* pair on the top_layer fixture -- all land " +
          "on the JS fallback path at open already). Verify on a real device instead.",
      },
      {
        name: "Combobox",
        supported: false,
        reason:
          "no naturally CSS-anchor-conforming context found for Combobox in this Chromium sandbox (checked " +
          "its own gallery page's demo instances and every clip-* pair on the top_layer fixture -- all land " +
          "on the JS fallback path at open already, which is itself the correctly-handled case, not this " +
          "gap). Combobox is the component the keyboard-overlap contract is about; real-device iOS Safari " +
          "verification is still needed for the exact 'conforming at open' shape (dev-docs/backlog.md row 28).",
      },
      {
        // dev-docs/backlog.md row 41(b). See this section's header doc's own
        // "Navbar" bullet above for the full reasoning and its caveat: this
        // is a structural inference from `ANCHORED_OVERLAYS`' own hover +
        // `reengage` shape for Navbar (identical to Tooltip/HoverCard's,
        // already execution-confirmed above), not an independent execution
        // run against Navbar itself in this environment.
        name: "Navbar",
        supported: false,
        reason:
          "same as Tooltip/HoverCard above -- ANCHORED_OVERLAYS opens Navbar via .hover() with a reengage " +
          "step after the keyboard-simulation resize, the same hover-driven shape that makes Tooltip/HoverCard " +
          "genuinely close via a real pointerleave when the resize moves the trigger out from under the " +
          "stationary synthetic cursor, before the tracking logic ever runs. Not independently confirmed by " +
          "execution for Navbar in this environment (dev-docs/backlog.md rows 36/41's own report) -- a future " +
          "execution pass should confirm or correct this.",
      },
      {
        // dev-docs/backlog.md row 72. Same structural inference as
        // the Navbar entry above (in `ANCHORED_OVERLAYS` for parts
        // (a)/(b), not in this `CONFORMING_CASES`
        // table): NavigationMenu opens the same way here as in
        // `ANCHORED_OVERLAYS` above -- `.hover()` on its trigger, with a
        // `reengage` step after the keyboard-simulation resize -- the
        // identical hover-driven shape that makes Tooltip/HoverCard/Navbar
        // genuinely close via a real pointerleave when the resize moves
        // the trigger out from under the stationary synthetic cursor,
        // before the tracking logic ever runs.
        name: "NavigationMenu",
        supported: false,
        reason:
          "same as Tooltip/HoverCard/Navbar above -- ANCHORED_OVERLAYS opens NavigationMenu via .hover() with " +
          "a reengage step after the keyboard-simulation resize, the same hover-driven shape that makes those " +
          "three genuinely close via a real pointerleave when the resize moves the trigger out from under the " +
          "stationary synthetic cursor, before the tracking logic ever runs. This is the same structural " +
          "inference the Navbar bullet above draws, not an independent execution run against the specific " +
          "'conforming at open' shape this test needs for NavigationMenu (it is skipped, like Navbar, by " +
          "test.skip(!kase.supported, ...) below, so this reason itself is never exercised by a normal run) " +
          "-- a future execution pass could confirm or correct it, the same standard the Navbar entry is held to.",
      },
    ];

    for (const kase of CONFORMING_CASES) {
      test(`${kase.name}: conforming at open, diverging later (anchor support removed after opening, then the keyboard opens)`, async ({
        page,
      }) => {
        test.skip(!kase.supported, !kase.supported ? kase.reason : "");
        if (!kase.supported) return;

        await page.setViewportSize(MOBILE_VIEWPORT);
        await kase.goto(page);
        await pinNearTop(page, kase.trigger(page));
        await kase.open(page);
        await expect(kase.content(page)).toBeVisible();

        // Confirm CSS Anchor Positioning was genuinely doing the work at
        // open (no inline `top` -- the fallback never engaged; same check
        // Rule 8 makes for its own CSS-anchor-path case above).
        const inlineTopAtOpen = await kase.content(page).evaluate((el) => (el as HTMLElement).style.top);
        expect(inlineTopAtOpen, `expected the CSS-anchor path to be active at open for ${kase.name}`).toBe("");

        await stripAnchorSupportsBlock(page);
        await simulateOnScreenKeyboard(page);

        // The direct, mechanism-level assertion: did anything re-measure at
        // all? If gap 2 were open the answer would be no -- no listener
        // is ever attached, since this content conformed (via CSS) at open --
        // so `style.top` stays exactly `""` forever, regardless of whether
        // this fixture's particular geometry also happens to overlap or
        // leave the viewport (see this section's header doc for why an
        // overlap/viewport check alone isn't sufficient here).
        const inlineTopAfter = await kase.content(page).evaluate((el) => (el as HTMLElement).style.top);
        expect(
          inlineTopAfter,
          `expected the fix's tracking to have re-measured and taken over once CSS Anchor Positioning ` +
            `stopped resolving mid-open for ${kase.name}, but style.top is still ${JSON.stringify(inlineTopAfter)}`,
        ).not.toBe("");

        const trigger = await rectOfLocator(kase.trigger(page));
        const content = await rectOfLocator(kase.content(page));
        const viewport = await viewportSize(page);
        const debug = JSON.stringify({ overlay: kase.name, trigger, content, viewport });
        expect(rectsIntersect(content, trigger), debug).toBe(false);
        expect(content.top, debug).toBeGreaterThanOrEqual(-EDGE_TOLERANCE);
        expect(content.bottom, debug).toBeLessThanOrEqual(viewport.height + EDGE_TOLERANCE);
      });
    }
  });
});

/**
 * Rule 12 — inline-axis shift (the `ColorPicker` popup on the home page's
 * widget masonry must not clip against the viewport's edge on a small
 * screen).
 *
 * Citation: same as Rule 11 above -- this repo's own anchored-overlay
 * placement contract (`[data-side]`/`[data-align]`, `use_anchor_position_
 * fallback`), not a normative WHATWG/W3C MUST. The platform primitive this
 * mirrors is W3C CSS Anchor Positioning's own collision-avoidance family
 * (`position-try-fallbacks`, <https://www.w3.org/TR/css-anchor-position-1/#fallback-var>),
 * of which this crate's `@supports` block declares only the flip members
 * (`flip-block`, `flip-inline`, Rules 5/6 above) -- no shift primitive, on
 * either engine.
 *
 * Why a shift is needed (see `use_anchor_position_fallback`'s doc, "Inline-axis shift",
 * `primitives/src/top_layer.rs`): flip only ever swaps `side`
 * to its own axis's opposite (block for top/bottom, inline for left/right).
 * A `side="top"`/`side="bottom"` placement's horizontal position comes
 * entirely from `align`, computed once against the trigger and never
 * checked against the viewport at all -- so a center-aligned overlay wider
 * than the room its trigger has on one side runs past that edge, on
 * an engine with no CSS Anchor Positioning (iOS 18 Safari; simulated here
 * exactly as Rule 11 does).
 *
 * Two fixture halves:
 *   - The component the rule is about: the widget-masonry `ColorPicker` on
 *     the home page (`/`), which sits near the *left* edge of a narrow
 *     viewport -- its `side="bottom"`/`align="center"` (defaults) content
 *     is wide enough to run a few pixels past the left edge there. Only the
 *     horizontal bounds are asserted for this case, deliberately: opening
 *     this specific component's popover independently grabs focus onto an
 *     internal drag-thumb (`ColorArea`'s `AreaThumb`, `color_picker.rs`),
 *     which drives a large, unrelated vertical scroll on this fixture
 *     regardless of the shift -- not something this rule is about
 *     (tracked separately, `dev-docs/backlog.md`).
 *   - Two of this file's own `edge-bottom-*` fixture cases (`Tooltip`,
 *     non-modal `Popover`) -- without the shift they land at
 *     `left: -192px`/`-194px` under the same simulation, a much larger
 *     margin than the ColorPicker case and free of that component's own
 *     scroll confound, so both axes are asserted here. `HoverCard` is
 *     included too for the same reason Rule 5 includes all three: it
 *     already stays within bounds here, so it is a passing regression
 *     guard, not a second discriminating case.
 */
test.describe("Rule 12 — inline-axis shift: a center-aligned overlay wider than the room its trigger has on one side stays within the viewport horizontally (no CSS Anchor Positioning engine)", () => {
  test("ColorPicker popover (home page widget masonry) stays within the viewport horizontally", async ({ page }) => {
    await page.setViewportSize(MOBILE_VIEWPORT);
    // dev-docs/backlog.md row 109: SSG lane -- `gotoHydrated`, see this
    // file's own `gotoFixture` doc.
    await gotoHydrated(page, `${BASE_URL}/?`, { timeout: NAV_TIMEOUT });
    await stripAnchorSupportsBlock(page);

    const trigger = page.getByRole("button", { name: /Color picker/i }).first(); // the home page's widget masonry, not a demo frame
    await trigger.scrollIntoViewIfNeeded();
    await trigger.evaluate((el) => (el as HTMLElement).click());
    const content = page.getByRole("dialog");
    await expect(content).toBeVisible();
    // Let the settle loop (`use_anchor_position_fallback`'s own
    // content-size-growth correction, see that function's doc) finish
    // before reading a final rect.
    await page.waitForTimeout(300);

    // Horizontal bounds only -- see this rule's header doc for why the
    // vertical axis is deliberately not asserted for this specific case.
    const rect = await readAtRest(content, (el) => {
      const r = (el as HTMLElement).getBoundingClientRect();
      return { left: r.left, right: r.right, width: r.width };
    });
    const debug = JSON.stringify({ rect, viewportWidth: MOBILE_VIEWPORT.width });
    expect(rect.left, debug).toBeGreaterThanOrEqual(-EDGE_TOLERANCE);
    expect(rect.right, debug).toBeLessThanOrEqual(MOBILE_VIEWPORT.width + EDGE_TOLERANCE);
  });

  const EDGE_BOTTOM_CASES: { name: string; triggerId: string; contentId: string; open: (page: Page) => Promise<void> }[] = [
    {
      name: "Tooltip",
      triggerId: "edge-bottom-tooltip-trigger",
      contentId: "edge-bottom-tooltip-content",
      open: async (page) => {
        await page.locator("#edge-bottom-tooltip-trigger").hover();
      },
    },
    {
      name: "HoverCard",
      triggerId: "edge-bottom-hovercard-trigger",
      contentId: "edge-bottom-hovercard-content",
      open: async (page) => {
        await page.locator("#edge-bottom-hovercard-trigger").hover();
      },
    },
    {
      name: "Popover (non-modal)",
      triggerId: "edge-bottom-popover-trigger",
      contentId: "edge-bottom-popover-content",
      open: async (page) => {
        await page.locator("#edge-bottom-popover-trigger").evaluate((el) => (el as HTMLElement).click());
      },
    },
  ];

  for (const kase of EDGE_BOTTOM_CASES) {
    test(`${kase.name} (near-left-edge, center-aligned) stays within the viewport horizontally`, async ({ page }) => {
      await page.setViewportSize(MOBILE_VIEWPORT);
      await gotoFixture(page);
      await stripAnchorSupportsBlock(page);
      await kase.open(page);
      await expect(page.locator(`#${kase.contentId}`)).toBeVisible();

      const content = await rectOf(page, `#${kase.contentId}`);
      const viewport = await viewportSize(page);
      const debug = JSON.stringify({ overlay: kase.name, content, viewport });
      expect(content.left, debug).toBeGreaterThanOrEqual(-EDGE_TOLERANCE);
      expect(content.right, debug).toBeLessThanOrEqual(viewport.width + EDGE_TOLERANCE);
    });
  }
});

/**
 * Rule 13 — fit-content width (a dropdown menu must not take the full page
 * width).
 *
 * Regression guard for the exact failure mode `top_layer::
 * anchored_content_attributes` (`primitives/src/top_layer.rs`) exists to
 * prevent: a duplicated `style` attribute on an
 * anchored-content leaf where SSR and the WASM client disagree about which
 * `style` wins (`dev-docs/conformance-harness.md` hydration-parity Rule 4). If
 * the client ever kept a caller/wrapper `style` in place of (rather than
 * folded with) this module's own `position-anchor`/`position: fixed`
 * declarations, the popover UA stylesheet's `[popover] { inset: 0; margin:
 * auto; }` centering-trap default (undone by the engine-injected
 * `:where(...)` reset in `ensure_anchor_positioning_styles`) or a stray
 * caller `width: 100%` would be free to stretch the content across the
 * whole viewport instead of shrink-wrapping its own children -- the
 * symptom. Every one of these components is authored `min-width`
 * only (no `width`), so a correctly-folded `style` always leaves the UA
 * `width: fit-content` default (or this fixture's own short menu-item/
 * option text) in charge, and the box stays well under the viewport.
 *
 * Uses the same `#clip-*-trigger`/`#clip-*-content` fixture pairs Rule 1
 * above already opens (one already-anchored instance per component is
 * enough; this rule checks a different property of the same open content,
 * not a new state) -- every anchored content on this fixture page in one
 * place, so a regression in any one of them fails loudly here rather than
 * only in that component's own, separate spec file.
 */
test.describe("Rule 13 — fit-content width (not full page width)", () => {
  const WIDTH_CASES: Array<{
    name: string;
    triggerId: string;
    contentId: string;
    open: (page: Page) => Promise<void>;
  }> = [
    {
      name: "Tooltip",
      triggerId: "clip-tooltip-trigger",
      contentId: "clip-tooltip-content",
      open: (page) => page.locator("#clip-tooltip-trigger").hover(),
    },
    {
      name: "HoverCard",
      triggerId: "clip-hovercard-trigger",
      contentId: "clip-hovercard-content",
      open: (page) => page.locator("#clip-hovercard-trigger").hover(),
    },
    {
      name: "Popover (non-modal)",
      triggerId: "clip-popover-trigger",
      contentId: "clip-popover-content",
      open: (page) => page.locator("#clip-popover-trigger").click(),
    },
    {
      name: "DropdownMenu",
      triggerId: "clip-dropdown-menu-trigger",
      contentId: "clip-dropdown-menu-content",
      open: (page) => page.locator("#clip-dropdown-menu-trigger").click(),
    },
    {
      name: "Menubar",
      triggerId: "clip-menubar-trigger",
      contentId: "clip-menubar-content",
      open: (page) => page.locator("#clip-menubar-trigger").click(),
    },
    {
      name: "Select",
      triggerId: "clip-select-trigger",
      contentId: "clip-select-content",
      open: (page) => page.locator("#clip-select-trigger").click(),
    },
    {
      name: "Combobox",
      triggerId: "clip-combobox-trigger",
      contentId: "clip-combobox-content",
      open: async (page) => {
        await page.locator("#clip-combobox-trigger").click();
        await page.keyboard.press("ArrowDown");
      },
    },
    {
      name: "Navbar",
      triggerId: "clip-navbar-trigger",
      contentId: "clip-navbar-content",
      open: (page) => page.locator("#clip-navbar-trigger").hover(),
    },
    {
      // dev-docs/backlog.md row 72.
      name: "NavigationMenu",
      triggerId: "clip-navigation-menu-trigger",
      contentId: "clip-navigation-menu-content",
      open: (page) => page.locator("#clip-navigation-menu-trigger").hover(),
    },
  ];

  for (const kase of WIDTH_CASES) {
    test(`${kase.name} content is well under the viewport width, not stretched full-page`, async ({ page }) => {
      await gotoFixture(page);
      await kase.open(page);
      await expect(page.locator(`#${kase.contentId}`)).toBeVisible();

      const content = await rectOf(page, `#${kase.contentId}`);
      const trigger = await rectOf(page, `#${kase.triggerId}`);
      const viewport = await viewportSize(page);
      const debug = JSON.stringify({ overlay: kase.name, content, trigger, viewport });
      // Fit-content, not full-page: well under the viewport width (60%
      // leaves generous room for a component's own min-width plus this
      // fixture's short fixed label/option text, while still catching a
      // stretched-to-100% regression by a wide margin).
      expect(content.width, debug).toBeLessThan(viewport.width * 0.6);
      // Still anchored near its own trigger horizontally, not detached to
      // the viewport's left edge the way an un-folded/lost `position-anchor`
      // binding would land it (`position: absolute; left: 0` from each
      // component's own pre-top-layer stylesheet fallback).
      expect(Math.abs(content.left - trigger.left), debug).toBeLessThan(viewport.width * 0.5);
    });
  }
});

/**
 * Rule 14 — trigger id resync: an overlay's `aria-labelledby` must resolve
 * to an element that actually exists, at the caller's own id where the
 * caller overrides one (dev-docs/backlog.md rows 36/41).
 *
 * Citation: WAI-ARIA's `aria-labelledby` is only well-formed when it refers
 * to an `id` present in the document -- the same normative property the
 * APG Menu Button pattern's "aria-labelledby set to a value that refers to
 * the menuitem or button that controls its display" already cites
 * (`oracle/tier1-apg/menu-roles.spec.ts`'s file header), applied here to
 * the *id-desync* failure mode rather than the *missing-labelledby-at-all*
 * one that file's calibration already covers.
 *
 * The hazard (row 36): `DropdownMenuContent`/`ContextMenuContent`/
 * `MenubarContent`/`SelectList`/`NavbarContent` (row 41) all label
 * themselves from an internally generated `trigger_id` context signal, so
 * a trigger that renders a caller-supplied `id` (exactly what this
 * fixture's `clip-*-trigger` ids are) through the caller-wins
 * `merge_attributes` pattern without writing it back into that signal
 * wins the *rendered* DOM id while the content's `aria-labelledby` keeps
 * pointing at the internally generated one nothing in the DOM carries: a
 * dangling reference. `SelectTrigger`'s case is axe-detectable (Rule
 * "Select listbox open" in the section above, via `aria-input-field-name`);
 * the other four are not (row 25: axe carries no rule that checks a bare
 * `role="menu"` container for an accessible name at all) -- this rule is
 * the direct, non-axe regression guard that catches all five the same way,
 * by checking the reference itself rather than relying on axe's coverage
 * gaps lining up.
 *
 * The construction: `SelectTrigger`/`DropdownMenuTrigger`/`ContextMenuTrigger`/
 * `MenubarTrigger`/`NavbarTrigger` feed their shared `trigger_id` context
 * signal straight into `use_id_or` alongside a typed `id` prop, mirroring `DialogTitle`'s
 * identical `use_id_or(ctx.dialog_labelledby, props.id)`
 * (`primitives/src/dialog.rs`) -- `use_id_or`'s own effect writes a caller
 * override back into that signal, so every reader of `trigger_id`
 * (`aria-labelledby` here, `use_refocus_on_close_unless`, the anchor-
 * position fallback) agrees with the actually-rendered id.
 *
 * Reuses the `#clip-*-trigger`/`#clip-*-content` fixture pairs above --
 * every one of these five already carries a caller-overridden trigger `id`
 * (`clip-select-trigger`, `clip-dropdown-menu-trigger`,
 * `clip-context-menu-trigger`, `clip-menubar-trigger`,
 * `clip-navbar-trigger`), which is exactly the condition row 36 needs to
 * manifest -- no extra fixture markup required. `Tooltip`/`HoverCard`/
 * `Popover`/`Combobox` are not included: none of them label their content
 * from a `trigger_id`-shaped context signal the way this family does
 * (`Tooltip`/`HoverCard`/`Popover` use `aria-describedby`/no default label
 * at all; `Combobox`'s listbox labelling is a distinct, already-covered
 * mechanism -- `oracle/tier1-apg/menu-roles.spec.ts`), so row 36's defect
 * class cannot arise for them.
 *
 * Each of the five cases below fails if the labelledby value resolves to
 * nothing, or to the stale internally-generated id rather than the
 * caller's own (dev-docs/backlog.md rows 36/41).
 */
test.describe("Rule 14 — aria-labelledby resolves to the caller-overridden trigger id", () => {
  const LABELLEDBY_CASES: Array<{
    name: string;
    triggerId: string;
    contentId: string;
    open: (page: Page) => Promise<void>;
  }> = [
    {
      name: "DropdownMenu",
      triggerId: "clip-dropdown-menu-trigger",
      contentId: "clip-dropdown-menu-content",
      open: (page) => page.locator("#clip-dropdown-menu-trigger").click(),
    },
    {
      name: "ContextMenu",
      triggerId: "clip-context-menu-trigger",
      contentId: "clip-context-menu-content",
      open: (page) => page.locator("#clip-context-menu-trigger").click({ button: "right" }),
    },
    {
      name: "Menubar",
      triggerId: "clip-menubar-trigger",
      contentId: "clip-menubar-content",
      open: (page) => page.locator("#clip-menubar-trigger").click(),
    },
    {
      name: "Select",
      triggerId: "clip-select-trigger",
      contentId: "clip-select-content",
      open: (page) => page.locator("#clip-select-trigger").click(),
    },
    {
      name: "Navbar",
      triggerId: "clip-navbar-trigger",
      contentId: "clip-navbar-content",
      open: (page) => page.locator("#clip-navbar-trigger").hover(),
    },
    {
      // dev-docs/backlog.md row 72: `NavigationMenuContent` labels
      // itself from `item_ctx.trigger_id` (`navigation_menu.rs`) and
      // `NavigationMenuTrigger` feeds a caller override into that
      // same signal via `use_id_or`, the identical construction row 36 gives
      // the other five components above, so this case holds for the same
      // reason: the content's `aria-labelledby` and the caller-overridden
      // trigger id agree.
      name: "NavigationMenu",
      triggerId: "clip-navigation-menu-trigger",
      contentId: "clip-navigation-menu-content",
      open: (page) => page.locator("#clip-navigation-menu-trigger").hover(),
    },
  ];

  for (const kase of LABELLEDBY_CASES) {
    test(`${kase.name}: content's aria-labelledby resolves to the trigger's own (caller-overridden) id`, async ({
      page,
    }) => {
      await gotoFixture(page);
      await kase.open(page);
      const content = page.locator(`#${kase.contentId}`);
      await expect(content).toBeVisible();

      const trigger = page.locator(`#${kase.triggerId}`);
      // The trigger itself must still render at the caller's own id --
      // otherwise the fixture stopped exercising the caller-wins case this
      // rule depends on, and every assertion below would vacuously pass.
      await expect(trigger, `${kase.name} trigger must render at the caller's own id`).toHaveCount(1);

      const labelledby = await content.getAttribute("aria-labelledby");
      expect(labelledby, `${kase.name} content must carry an aria-labelledby`).not.toBeNull();
      // The direct row-36 regression check: the id the content labels
      // itself by must be the trigger's own *actual, caller-overridden*
      // id, not a stale internally generated one nothing in the DOM has.
      expect(
        labelledby,
        `${kase.name}: aria-labelledby must equal the caller's own trigger id ("${kase.triggerId}"), ` +
          `not an internally generated one left dangling by the caller override`,
      ).toBe(kase.triggerId);

      // Non-dangling: the id aria-labelledby names must resolve to
      // exactly one real element in the document (the fix must have
      // resynced the signal, not merely rendered a plausible-looking
      // string).
      const resolvedCount = await page.locator(`#${labelledby}`).count();
      expect(
        resolvedCount,
        `${kase.name}: aria-labelledby="${labelledby}" must resolve to exactly one element in the document`,
      ).toBe(1);
    });
  }
});

/**
 * Rule 15 — vertical shift + size clamp (dev-docs/backlog.md row 10 residual
 * scope: "shift/size clamping").
 *
 * Citation: same as Rules 11/12 above -- this repo's own anchored-overlay
 * placement contract (`use_anchor_position_fallback`,
 * `primitives/src/top_layer.rs`), not a normative WHATWG/W3C MUST. See that
 * function's doc, "Vertical shift + size clamp", for the full
 * construction this rule guards: CSS Anchor Positioning's own
 * collision-avoidance family (`position-try-fallbacks`) has no primitive
 * for either half of it -- no shift-along-an-axis keyword (the same gap
 * Rule 12 already documents for the horizontal axis) and no size-clamp
 * primitive at all (nothing W3C-shipped shrinks an anchored box to fit a
 * short viewport the way Floating UI's `size()` middleware does in
 * userland JS) -- so both stay JS-fallback-only, mirroring Rule 12's own
 * engine simulation (`stripAnchorSupportsBlock`) with a *short*, not
 * narrow, custom viewport: this rule is about the vertical axis, not the
 * horizontal one Rule 12 already covers.
 *
 * Fixture: the "Light dismiss, Escape, and stacking" section's
 * `stack-popover-trigger`/`stack-popover-content` pair (Rules 2-4 above),
 * deliberately NOT the `clip-*-content` family Rules 1/11/13/14 reuse:
 * every one of those carries an inline `min-height: 100px` for Rule 1's own
 * purposes, and a `min-height` on the SAME element this rule's
 * `max-height` clamp also targets would win the CSS min/max conflict
 * outright (CSS2.1 §10.7: a larger `min-height` overrides a smaller
 * `max-height` on the same box) -- silently defeating the very clamp this
 * rule exists to check, not a false pass but a wrong one. `stack-popover-
 * content` carries no `min-height` of its own
 * (`preview/src/components/popover/style.css`), so nothing here competes
 * with the engine's own inline override.
 *
 * A synthetic 500px-tall child is appended to the content *after* it opens
 * -- well past what the short custom viewport below can hold either way it
 * might be positioned, and deterministic regardless of this fixture's own
 * text/padding metrics, unlike relying on a fixed `min-height` (as the
 * `clip-*` fixtures do for Rule 1) to just barely exceed a computed
 * available-height figure derived from font/padding sizes this test does
 * not control.
 *
 * Re-measurement: `use_anchor_position_fallback`'s tracking listeners
 * (`window`'s `resize`/`scroll`, "Scroll/resize tracking" in that
 * function's doc) re-run `reposition()` on *any* fired event, synthetic or
 * real, and `usingFallback` is sticky once `stripAnchorSupportsBlock` has
 * forced the very first `reposition()` call onto the fallback path (Rules
 * 11/12's own precedent) -- so a plain
 * `window.dispatchEvent(new Event('resize'))`, fired after appending the
 * filler child, is enough to force a fresh measurement that sees the new
 * (much taller) content, with no real second viewport resize needed.
 */
test.describe("Rule 15 — vertical shift + size clamp: an overlay taller than the viewport is clamped to fit and made scrollable, not left to overflow off-screen", () => {
  test("Popover content taller than a short viewport is capped, made scrollable, and stays within the viewport vertically", async ({
    page,
  }) => {
    // Short, not narrow (unlike Rule 12's MOBILE_VIEWPORT) -- this rule is
    // about the vertical axis. Wide enough that this fixture's normal
    // width-driven layout (flex-wrap rows, `#clip-box`, etc.) is
    // unaffected; the exact width is otherwise unconstrained by anything
    // this rule checks.
    const SHORT_VIEWPORT = { width: 390, height: 400 };
    await page.setViewportSize(SHORT_VIEWPORT);
    await gotoFixture(page);
    await stripAnchorSupportsBlock(page);

    // `.evaluate((el) => el.click())`, not Playwright's own `.click()` --
    // same technique this file already uses for a fixed-position trigger a
    // small custom viewport might otherwise intercept behind the sticky
    // nav (see the `edge-bottom-popover-trigger` case above) -- bypasses
    // that actionability/visibility concern entirely, which matters here
    // since `SHORT_VIEWPORT` leaves little headroom below the nav.
    // Row 210 (exclusion zone): the panel takes the room BESIDE its trigger and never
    // slides over it, so the trigger must be on screen for "within the viewport" to be
    // possible at all (a trigger scrolled out of view keeps its panel beside it, out of view).
    // (Scrolled AFTER the open: with the CSS-anchor block stripped the content briefly sits in
    // flow and the open itself moves the page, so an earlier scroll does not hold.)
    await page.locator("#stack-popover-trigger").evaluate((el) => (el as HTMLElement).click());
    const content = page.locator("#stack-popover-content");
    await expect(content).toBeVisible();
    await page.locator("#stack-popover-trigger").evaluate((el) => el.scrollIntoView({ block: "center" }));

    // Grow the content well past what SHORT_VIEWPORT can hold (392px of
    // room once the engine's own 4px EDGE_MARGIN is subtracted from both
    // edges -- see this rule's header doc for why a plain child, not
    // `min-height` on the content element itself, is used here).
    // `flexShrink: "0"` matters, not decoration: `.dx-popover-content` is
    // `display: flex; flex-direction: column` (`popover/style.css`), and a
    // flex item's default `flex-shrink: 1` would otherwise let the
    // flexbox layout algorithm shrink this filler back down to fit inside
    // the engine's own `max-height` clamp -- exactly defeating the
    // overflow this test needs to exist for `scrollHeight >
    // clientHeight` to hold below.
    await content.evaluate((el) => {
      const filler = document.createElement("div");
      filler.id = "rule15-filler";
      filler.style.height = "500px";
      filler.style.flexShrink = "0";
      el.appendChild(filler);
    });
    // Force a fresh `reposition()` measurement -- see header doc,
    // "Re-measurement."
    await page.evaluate(() => window.dispatchEvent(new Event("resize")));
    // rAF-throttled, same wait shape as Rule 8/11's identical tracking waits.
    await page.waitForTimeout(300);

    const rect = await rectOf(page, "#stack-popover-content");
    const metrics = await content.evaluate((el) => {
      const style = getComputedStyle(el);
      return {
        offsetHeight: (el as HTMLElement).offsetHeight,
        clientHeight: (el as HTMLElement).clientHeight,
        scrollHeight: (el as HTMLElement).scrollHeight,
        overflowY: style.overflowY,
      };
    });
    const viewport = await viewportSize(page);
    // Mirrors `use_anchor_position_fallback`'s own `EDGE_MARGIN` constant
    // (`primitives/src/top_layer.rs`) -- kept as a literal here, not
    // imported, since this file has no access to that Rust module's
    // constants and duplicating a single small number is clearer than
    // scraping it out at runtime.
    const EDGE_MARGIN = 4;
    // The room on the side of the trigger the engine chose (the larger one, since neither fits
    // 550px): from the trigger's edge, past the gap, to the viewport edge less the margin.
    const triggerRect = await rectOf(page, "#stack-popover-trigger");
    const expectedAvailableHeight = Math.max(
      triggerRect.top - EDGE_MARGIN,
      SHORT_VIEWPORT.height - EDGE_MARGIN - triggerRect.bottom,
    );
    const debug = JSON.stringify({ rect, metrics, viewport, expectedAvailableHeight, triggerRect });

    // Position clamp (gap 1): fully within the viewport vertically -- not
    // run off either edge the way an unclamped 500px-tall box, positioned
    // relative to the trigger's own (unrelated, in-flow) location, would
    // be on a 400px-tall viewport.
    expect(rect.top, debug).toBeGreaterThanOrEqual(-EDGE_TOLERANCE);
    expect(rect.bottom, debug).toBeLessThanOrEqual(viewport.height + EDGE_TOLERANCE);
    // ...and never over the trigger (the anchor is an exclusion zone).
    expect(rect.bottom <= triggerRect.top + EDGE_TOLERANCE || rect.top >= triggerRect.bottom - EDGE_TOLERANCE, debug).toBe(true);

    // Size clamp (gap 2): the rendered box itself is capped at the engine's
    // computed available height, not merely "happens to end up on-screen"
    // by some other coincidence.
    expect(metrics.offsetHeight, debug).toBeLessThanOrEqual(expectedAvailableHeight + EDGE_TOLERANCE);
    expect(metrics.offsetHeight, debug).toBeGreaterThan(0);
    // -- AND made scrollable, not merely truncated/hidden: `overflow-y`
    // actually engaged, and there is genuinely more content
    // (`scrollHeight`, the filler's full 500px plus the original text) than
    // the visible, capped box (`clientHeight`) can show -- the literal
    // "clamped and made scrollable, not left to overflow off-screen"
    // contract this rule's title states.
    expect(metrics.overflowY, debug).toBe("auto");
    expect(metrics.scrollHeight, debug).toBeGreaterThan(metrics.clientHeight);
  });
});

/**
 * Rule 16 — shift and size clamp on a CONFORMING engine (dev-docs/backlog.md
 * row 10, the "JS-fallback shift" follow-up).
 *
 * Rules 12 and 15 prove the cross-axis shift and the size clamp only under
 * `stripAnchorSupportsBlock`, i.e. on an engine with no CSS Anchor
 * Positioning. The other half is row 10's gap: the
 * `@supports` contract declares `position-try-fallbacks: flip-block,
 * flip-inline` and nothing else, CSS has no shift primitive and no
 * shrink-to-fit primitive, so on an engine that DOES resolve `anchor()`
 * (this sandbox's Chromium, Chrome/Edge 125+, FF 147+, Safari 26+) a
 * center-aligned panel wider than its trigger's room runs past the
 * viewport edge, and a panel taller than the viewport overflows it, unless
 * `use_anchor_position_fallback` hands the call to its JS path. These are
 * the same fixtures and assertions as Rules 12 and 15, with the engine's
 * own anchor positioning left in place, so they pin that hand-off (row
 * 210's exclusion-zone check: a CSS placement outside the visual viewport
 * goes to the JS path). With that
 * check mutated off, Tooltip and Popover land at `left: -85px`/`-73px` and
 * the filled Popover reaches `bottom: 806px` on a 400px viewport (red).
 * HoverCard stays in bounds either way, a guard as in Rule 12.
 */
test.describe("Rule 16 — shift and size clamp hold on a conforming CSS Anchor Positioning engine too", () => {
  const CASES: { name: string; contentId: string; open: (page: Page) => Promise<void> }[] = [
    {
      name: "Tooltip",
      contentId: "edge-bottom-tooltip-content",
      open: async (page) => {
        await page.locator("#edge-bottom-tooltip-trigger").hover();
      },
    },
    {
      name: "HoverCard",
      contentId: "edge-bottom-hovercard-content",
      open: async (page) => {
        await page.locator("#edge-bottom-hovercard-trigger").hover();
      },
    },
    {
      name: "Popover (non-modal)",
      contentId: "edge-bottom-popover-content",
      open: async (page) => {
        await page.locator("#edge-bottom-popover-trigger").evaluate((el) => (el as HTMLElement).click());
      },
    },
  ];

  for (const kase of CASES) {
    test(`${kase.name} (near-left-edge, center-aligned) stays within the viewport horizontally`, async ({ page }) => {
      await page.setViewportSize(MOBILE_VIEWPORT);
      await gotoFixture(page);
      await kase.open(page);
      await expect(page.locator(`#${kase.contentId}`)).toBeVisible();
      // The settle loop and the rAF-throttled tracking, as Rule 12.
      await page.waitForTimeout(300);

      const content = await rectOf(page, `#${kase.contentId}`);
      const viewport = await viewportSize(page);
      const debug = JSON.stringify({ overlay: kase.name, content, viewport });
      expect(content.left, debug).toBeGreaterThanOrEqual(-EDGE_TOLERANCE);
      expect(content.right, debug).toBeLessThanOrEqual(viewport.width + EDGE_TOLERANCE);
      expect(content.top, debug).toBeGreaterThanOrEqual(-EDGE_TOLERANCE);
      expect(content.bottom, debug).toBeLessThanOrEqual(viewport.height + EDGE_TOLERANCE);
    });
  }

  test("Popover content taller than a short viewport is capped, made scrollable, and stays within the viewport vertically", async ({
    page,
  }) => {
    const SHORT_VIEWPORT = { width: 390, height: 400 };
    await page.setViewportSize(SHORT_VIEWPORT);
    await gotoFixture(page);
    await page.locator("#stack-popover-trigger").evaluate((el) => el.scrollIntoView({ block: "center" }));
    await page.locator("#stack-popover-trigger").evaluate((el) => (el as HTMLElement).click());
    const content = page.locator("#stack-popover-content");
    await expect(content).toBeVisible();

    // Same filler as Rule 15 (see its header doc for `flexShrink`).
    await content.evaluate((el) => {
      const filler = document.createElement("div");
      filler.id = "rule16-filler";
      filler.style.height = "500px";
      filler.style.flexShrink = "0";
      el.appendChild(filler);
    });
    await page.evaluate(() => window.dispatchEvent(new Event("resize")));
    await page.waitForTimeout(300);

    const rect = await rectOf(page, "#stack-popover-content");
    const metrics = await content.evaluate((el) => {
      const style = getComputedStyle(el);
      return {
        offsetHeight: (el as HTMLElement).offsetHeight,
        clientHeight: (el as HTMLElement).clientHeight,
        scrollHeight: (el as HTMLElement).scrollHeight,
        overflowY: style.overflowY,
      };
    });
    const viewport = await viewportSize(page);
    const EDGE_MARGIN = 4;
    const triggerRect = await rectOf(page, "#stack-popover-trigger");
    const expectedAvailableHeight = Math.max(
      triggerRect.top - EDGE_MARGIN,
      viewport.height - EDGE_MARGIN - triggerRect.bottom,
    );
    const debug = JSON.stringify({ rect, metrics, viewport, expectedAvailableHeight, triggerRect });

    expect(rect.top, debug).toBeGreaterThanOrEqual(-EDGE_TOLERANCE);
    expect(rect.bottom, debug).toBeLessThanOrEqual(viewport.height + EDGE_TOLERANCE);
    expect(rect.bottom <= triggerRect.top + EDGE_TOLERANCE || rect.top >= triggerRect.bottom - EDGE_TOLERANCE, debug).toBe(true);
    expect(metrics.offsetHeight, debug).toBeLessThanOrEqual(expectedAvailableHeight + EDGE_TOLERANCE);
    expect(metrics.overflowY, debug).toBe("auto");
    expect(metrics.scrollHeight, debug).toBeGreaterThan(metrics.clientHeight);
  });
});
