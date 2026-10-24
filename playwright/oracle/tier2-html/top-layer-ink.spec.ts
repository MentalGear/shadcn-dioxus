/**
 * ORACLE: tier 2 (HTML/CSS) — top-layer text color (`[popover]`/`<dialog>`
 * UA `color: CanvasText` beats DOM-tree inheritance).
 *
 * ## The bug class (this file is its regression oracle)
 *
 * Both the `[popover]` and `<dialog>` user-agent stylesheets set `color:
 * CanvasText` **directly on the element**:
 *   - WHATWG HTML's rendering chapter defines the popover UA rules:
 *     <https://html.spec.whatwg.org/multipage/rendering.html#the-popover-attribute-2>
 *   - The `<dialog>` element's own UA rules (same chapter):
 *     <https://html.spec.whatwg.org/multipage/rendering.html#flow-content-3>
 *     (Chromium's concrete `html.css`/`user-agent-content.css` -- not a
 *     spec text, but the shipped instance of the same WHATWG-mandated
 *     defaults -- literally has `[popover], dialog { color: CanvasText;
 *     ... }`; `CanvasText` itself is a CSS Color Module Level 4 system
 *     color keyword: <https://www.w3.org/TR/css-color-4/#css-system-colors>.)
 *
 * A **directly-cascaded** declaration on an element always wins over an
 * **inherited** one, regardless of specificity or source order -- the
 * cascade's "winning declaration" algorithm only ever falls through to an
 * ancestor's inherited value when the element itself has *no* declared
 * value for the property at all (CSS Cascading and Inheritance Level 4,
 * "the cascade": <https://www.w3.org/TR/css-cascade-4/#cascading>, and
 * "specified value"/inheritance: <https://www.w3.org/TR/css-cascade-4/
 * #inheriting>). Author origin already beats user-agent origin outright
 * (`#cascade-origin`), so any component's *own* `color` rule for its
 * top-layer content already wins with no help needed -- but a top-layer
 * wrapper that declares no `color` of its own falls straight to the UA
 * default instead of quietly inheriting the app's ink token
 * (`--dx-foreground`) the way every other element on the page does.
 *
 * ## The construction under test
 *
 * `primitives/src/top_layer.rs`'s `ensure_top_layer_ink_styles` installs a
 * pair of engine-injected, zero-specificity baselines:
 *
 *   :where([popover]), :where(dialog) { color: inherit; }
 *   :where([popover]) :where(button, input, select, textarea),
 *   :where(dialog) :where(button, input, select, textarea) { color: inherit; }
 *
 * The first fixes the promoted element itself; the second closes the
 * identical defect one DOM level deeper -- native form controls carry their
 * own direct UA `color` too, so the first rule's `inherit` cannot reach one
 * nested inside promoted content (see `date_picker`'s case below).
 * `:where()` contributes zero specificity in both, so neither ever does
 * more than fill a gap where nothing else is declared -- each can
 * out-cascade the UA default (author origin always beats user-agent, at any
 * specificity) but can never out-specificity-fight a component's own
 * `color` rule. `Tooltip`'s `.dx-tooltip-content` (which sets its own,
 * intentionally *inverted* `color: var(--dx-background)` against its dark
 * background) is the load-bearing negative case this file checks: it must
 * keep winning, not fall back to the shared rule's `inherit`.
 *
 * See that function's own doc (`primitives/src/top_layer.rs`) for the full
 * proof that every top-layer consumer in this crate reaches the fix,
 * through one of exactly three hooks (`use_popover_sync`,
 * `use_popover_shown_while_mounted`, `use_dialog_open_driver`) -- this file
 * is the black-box half of that proof: it opens every one of those
 * consumers as an end user would and asserts the *painted* result.
 *
 * ## Method: assert against the resolved token, not a hardcoded literal
 *
 * `resolvedColor()` below reads `--dx-foreground` (and, for the
 * Tooltip negative case, `--dx-background`) the only way a CSS custom
 * property can be read *as a color*: by assigning it to a probe element's
 * `color` and reading that element's own `getComputedStyle().color` back
 * (`getPropertyValue('--dx-foreground')` alone would return the raw,
 * unresolved token text -- literally `"var(--foreground, var(--light, oklch(14.5% 0 0deg))
 * var(--dark, oklch(98.5% 0 0deg)))"` per `preview/assets/dx-components-theme.css`, not a color). Every
 * assertion below compares a surface's own computed `color` against that
 * resolved value, so this file survives a theme retune untouched.
 *
 * `--dx-foreground` is set once, as `color`, on `body`
 * (`preview/assets/main.css`) -- and top-layer promotion (`showPopover()`/
 * `showModal()`) never moves an element in the *DOM tree*, only how it is
 * *painted*; CSS inheritance follows the DOM tree, not paint order. So
 * `color: inherit` on a promoted element resolves through its real DOM
 * ancestors back to `body`'s declaration regardless of which visual layer
 * it paints in -- which is also why this fix applies equally to a fully
 * themed component (`.dx-menubar-content`) and to a bare, unthemed
 * primitive (the `top_layer` fixture's raw `PopoverContent`, used below for
 * the non-modal `Popover` arm): neither declares `color` itself, both climb
 * the same DOM tree to the same `body` rule.
 *
 * ## Coverage
 *
 * Every top-layer surface, plus several *masked* siblings kept as living
 * proof the baseline doesn't regress an already-correct case: `.dx-navbar-item`/`.dx-toast-title`/`.dx-dialog`/
 * `.dx-alert-dialog`/`.dx-sheet` already declare their own `color: var(
 * --dx-foreground)`; `.dx-tooltip-content` declares its own
 * *intentionally different* `color: var(--dx-background)`; `.dx-select-
 * list`/`.dx-combobox-list` declare their own *intentionally different*
 * `color: var(--secondary-color-1)` (the ramp token row 111's popover role replaced). Every one of these is author-origin
 * and wins over the UA default with no help from the baseline -- each
 * is asserted against its OWN actual token (`expectedVar`), not assumed to
 * be the general ink token, so a real regression in any of them still
 * fails this file:
 *   menubar, navbar, dropdown_menu, context_menu, popover (non-modal arm,
 *   unthemed via the `top_layer` fixture's raw `PopoverContent`, and themed via
 *   the default `/component/popover/` demo, since `PopoverRootProps::is_modal`
 *   defaults `false` (dev-docs/backlog.md row 169)), popover (modal arm,
 *   the explicit `variant=modal` demo),
 *   hover_card, tooltip, date_picker, select, combobox, dialog,
 *   alert_dialog, sheet, toast. Every one of these opens cleanly in this
 *   harness. (`date_picker`'s text case is a real `<button>`, exercising
 *   `ensure_top_layer_ink_styles`'s second, form-control-scoped rule rather
 *   than the wrapper rule -- see that surface's own comment below.)
 *
 * Each surface is checked in both `colorScheme: 'light'` and `colorScheme:
 * 'dark'` (a separate `test.describe` per scheme, sharing one `run()`) --
 * dark is where this bug is worst (UA `CanvasText`
 * resolves near-white-on-dark in dark mode on some engines, but Chromium's
 * concrete default is a fixed opaque black in both schemes, so the visible
 * failure mode here is "black wrapper text on a dark themed background").
 */

import { test, expect } from "../../fixtures";
import { type Page, type Locator } from "@playwright/test";
import { BASE_URL } from "../../base-url";
import { gotoHydrated } from "../../hydration";
import { demoFrame, demoRole } from "../../demo-frame";

const NAV_TIMEOUT = 20 * 60 * 1000; // first run compiles the app

// The page ink: `body { color: var(--dx-foreground) }` (`preview/assets/main.css`). Read as a token at
// runtime by `resolvedColor`, never as a literal, so a retune of the ink (row 231 moved it from the
// ramp's `--secondary-color-4` #111 to shadcn's `--foreground`, and row 373 from #000 to oklch(0.145)) cannot stale this file again.
const INK_TOKEN = "--dx-foreground";

const url = (name: string) => `${BASE_URL}/component/${name}/`;

// dev-docs/backlog.md row 109: this suite clicks/hovers triggers right
// after navigating, on the SSG lane -- `gotoHydrated` so those interactions
// can't land before hydration attaches listeners.
async function goto(page: Page, name: string) {
  await gotoHydrated(page, url(name), { timeout: NAV_TIMEOUT });
}

/**
 * Reads a CSS custom property's *resolved color* -- the only way to read
 * one as a color at all -- by assigning it to a throwaway probe element's
 * `color` and reading that element's own computed style back. See this
 * file's header doc, "Method," for why `getPropertyValue` alone cannot be
 * used here.
 */
async function resolvedColor(page: Page, cssVar: string): Promise<string> {
  return page.evaluate((v) => {
    const probe = document.createElement("span");
    probe.style.color = `var(${v})`;
    document.body.appendChild(probe);
    const color = getComputedStyle(probe).color;
    probe.remove();
    return color;
  }, cssVar);
}

type Surface = {
  name: string;
  /** Navigate and drive the surface open. */
  open: (page: Page) => Promise<void>;
  /** The element that actually carries `[popover]` or is the `<dialog>` itself. */
  wrapper: (page: Page) => Locator;
  /** A text-bearing descendant (or the wrapper itself, when text is a direct child). */
  text: (page: Page) => Locator;
  /**
   * The CSS custom property this surface's text should resolve to.
   * Defaults to the app's ink token; Tooltip is the sole, deliberate
   * exception (its own inverted `color` must keep winning).
   */
  expectedVar?: string;
};

const SURFACES: Surface[] = [
  {
    name: "menubar",
    open: async (page) => {
      await goto(page, "menubar");
      await demoRole(page, "menuitem", 0).click(); // menubar demo: File, Edit
      await expect(page.locator('.dx-menubar-content[data-state="open"]')).toBeVisible();
    },
    wrapper: (page) => page.locator('.dx-menubar-content[data-state="open"]'),
    text: (page) => page.locator('.dx-menubar-content[data-state="open"]').getByRole("menuitem").first(),
  },
  {
    name: "navbar",
    open: async (page) => {
      await goto(page, "navbar");
      await demoRole(page, "menuitem", 0).hover(); // navbar demo: Inputs, Information, Home
      await expect(page.locator('.dx-navbar-content[data-state="open"]')).toBeVisible();
    },
    wrapper: (page) => page.locator('.dx-navbar-content[data-state="open"]'),
    text: (page) => page.locator('.dx-navbar-content[data-state="open"]').getByRole("menuitem").first(),
  },
  {
    name: "dropdown_menu",
    open: async (page) => {
      await goto(page, "dropdown_menu");
      await demoRole(page, "button").click();
      await expect(page.locator('.dx-dropdown-menu-content[data-state="open"]')).toBeVisible();
    },
    wrapper: (page) => page.locator('.dx-dropdown-menu-content[data-state="open"]'),
    text: (page) =>
      page.locator('.dx-dropdown-menu-content[data-state="open"] .dx-dropdown-menu-item').first(),
  },
  {
    name: "context_menu",
    open: async (page) => {
      await goto(page, "context_menu");
      await demoRole(page, "button").click({ button: "right" });
      await expect(page.locator('.dx-context-menu-content[data-state="open"]')).toBeVisible();
    },
    wrapper: (page) => page.locator('.dx-context-menu-content[data-state="open"]'),
    text: (page) =>
      page.locator('.dx-context-menu-content[data-state="open"] .dx-context-menu-item').first(),
  },
  {
    // Non-modal `Popover` arm (`popover="auto"`, not a modal `<dialog>`).
    // The `top_layer` fixture's `clip-popover-*` instance is a real,
    // deliberately *unthemed* `is_modal: false` `PopoverRoot`/`PopoverContent`,
    // which doubles as proof this fix works with zero component-authored CSS
    // in play at all, relying only on the DOM-tree inheritance chain up to
    // `body`. The themed default demo is the next entry.
    name: "popover (non-modal arm, unthemed)",
    open: async (page) => {
      await goto(page, "top_layer");
      await page.locator("#clip-popover-trigger").click();
      await expect(page.locator("#clip-popover-content")).toBeVisible();
    },
    wrapper: (page) => page.locator("#clip-popover-content"),
    text: (page) => page.locator("#clip-popover-content"),
  },
  {
    // Themed non-modal `Popover` (the primitive's default, row 169): `popover="auto"`
    // through `use_popover_shown_while_mounted`.
    name: "popover (non-modal arm, themed default)",
    open: async (page) => {
      await goto(page, "popover");
      await demoRole(page, "button").click();
      await expect(page.locator('.dx-popover-content[data-state="open"]')).toBeVisible();
    },
    wrapper: (page) => page.locator('.dx-popover-content[data-state="open"]'),
    text: (page) => page.locator(".dx-popover-content .dx-popover-title"),
  },
  {
    // Modal `Popover` arm (`is_modal: true`, the `modal` demo): a real `<dialog>` +
    // `showModal()` on the web arm (native-dialog engine migration) -- reaches the fix
    // through `use_dialog_open_driver`, not `use_popover_sync`.
    name: "popover (modal arm)",
    open: async (page) => {
      // Every variant has its own `#component-preview-frame-<variant>` on the canonical page (`demoRole(.., "modal")`);
      // `/component/popover/variant=modal&` (query text glued to a path route) is a 404 that never sets data-hydrated.
      await goto(page, "popover");
      await demoRole(page, "button", 0, "modal").click();
      await expect(page.locator('.dx-popover-content[data-state="open"]')).toBeVisible();
    },
    wrapper: (page) => page.locator('.dx-popover-content[data-state="open"]'),
    // The demo's "Dimensions" heading (`PopoverTitle`, `.dx-popover-title`) in shadcn's Dimensions form. The
    // heading has to pick up its class's declared color (not inherited); asserted against the shared ink token
    // here since that's also `.dx-popover-title`'s own declared value.
    text: (page) => page.locator(".dx-popover-content .dx-popover-title"),
  },
  {
    name: "hover_card",
    open: async (page) => {
      await goto(page, "hover_card");
      await demoRole(page, "button").hover();
      await expect(page.locator(".dx-hover-card-content")).toBeVisible();
    },
    wrapper: (page) => page.locator(".dx-hover-card-content"),
    text: (page) => page.locator(".dx-hover-card-content div").first(),
  },
  {
    // The load-bearing negative case: the tooltip is the theme's inverted floating surface
    // (`[data-dx-surface="floating"][data-dx-tone="inverted"] { color: var(--dx-background) }`, on a
    // `--dx-foreground` fill), so its ink is the page background token -- the shared `:where()` rule
    // must never override it. `expectedVar` below is `--dx-background`, not the ink token, on purpose;
    // it is READ from the page (`resolvedColor`), never a literal, so a palette change cannot break it.
    name: "tooltip",
    open: async (page) => {
      await goto(page, "tooltip");
      await demoFrame(page).locator(".dx-tooltip-trigger").hover();
      await expect(page.getByRole("tooltip")).toBeVisible();
    },
    wrapper: (page) => page.locator(".dx-tooltip-content"),
    text: (page) => page.locator(".dx-tooltip-content p").first(),
    expectedVar: "--dx-background",
  },
  {
    // Wrapper: `.dx-date-picker-popover-content`, a real `[popover]`
    // element with no `color` of its own -- this surface's direct instance
    // of the bug. Text: a calendar day-grid cell -- a real `<button>`, so
    // it exercises the *second*, form-control-scoped `:where()` rule
    // (`ensure_top_layer_ink_styles`'s own doc, "one level deeper"), not
    // just the wrapper rule. Both matter here specifically: `Calendar`'s own
    // stylesheet is not loaded on this route (`document.styleSheets` has no
    // `calendar` entry), so without the form-control rule these buttons
    // would have no author `color` of any kind to fall back to, only the UA
    // default this file exists to catch.
    name: "date_picker",
    open: async (page) => {
      await goto(page, "date_picker");
      await demoRole(page, "button").click();
      await expect(page.locator(".dx-date-picker-popover-content").first()).toBeVisible();
    },
    wrapper: (page) => page.locator(".dx-date-picker-popover-content").first(),
    text: (page) => page.locator('.dx-calendar-grid-cell[data-month="current"]').first(),
  },
  {
    // `.dx-select-list` declares its own `color: var(--dx-popover-foreground)`
    // directly (author origin, already winning over the UA default with no
    // help needed from this file's fix). Row 111 moved it from the ramp's
    // `--secondary-color-1` to the popover role, which resolves to
    // `--dx-popover-foreground`, so `expectedVar` names that
    // token; it is still the surface's OWN declared colour, not an instance
    // of the bug. This case exists
    // to prove the shared `:where()` rule leaves it alone, the same
    // load-bearing negative shape as the Tooltip case above.
    name: "select",
    open: async (page) => {
      await goto(page, "select");
      // `SelectTrigger` renders `role="combobox"`, not an implicit button
      // (`dev-docs/backlog.md` row 8); a `button` locator never matches and
      // hangs the click (and the test) until the 5-minute ceiling.
      await demoRole(page, "combobox").click();
      await expect(page.locator('.dx-select-list[data-state="open"]')).toBeVisible();
    },
    wrapper: (page) => page.locator('.dx-select-list[data-state="open"]'),
    text: (page) => page.locator('.dx-select-list[data-state="open"]').getByRole("option").first(),
    expectedVar: "--dx-popover-foreground",
  },
  {
    // Same shape as `select` immediately above: `.dx-combobox-list` also
    // declares its own `color: var(--dx-foreground)` directly
    // (the popover role, row 111; was `--secondary-color-1`).
    name: "combobox",
    open: async (page) => {
      await goto(page, "combobox");
      const input = demoRole(page, "combobox");
      await input.focus();
      await page.keyboard.press("ArrowDown");
      await expect(page.locator('.dx-combobox-list[data-state="open"]')).toBeVisible();
    },
    wrapper: (page) => page.locator('.dx-combobox-list[data-state="open"]'),
    // The first option is highlighted on open and so carries `--dx-accent-foreground`
    // (its own, deliberate colour, row 111); the first un-highlighted one inherits the list's.
    text: (page) =>
      page.locator('.dx-combobox-list[data-state="open"] .dx-combobox-option:not([data-highlighted="true"])').first(),
    expectedVar: "--dx-popover-foreground",
  },
  {
    name: "dialog",
    open: async (page) => {
      await goto(page, "dialog");
      await demoRole(page, "button").click();
      await expect(page.getByRole("dialog")).toBeVisible();
    },
    wrapper: (page) => page.getByRole("dialog"),
    text: (page) => page.locator(".dx-dialog-title"),
  },
  {
    name: "alert_dialog",
    open: async (page) => {
      await goto(page, "alert_dialog");
      await demoRole(page, "button").click();
      await expect(page.getByRole("alertdialog")).toBeVisible();
    },
    wrapper: (page) => page.getByRole("alertdialog"),
    text: (page) => page.locator(".dx-alert-dialog-title"),
  },
  {
    // `[data-slot="sheet-root"]`/`.dx-sheet-root` is NOT the promoted
    // element -- `Sheet` (`preview/src/components/sheet/component.rs`)
    // passes that class to `dialog::DialogRoot`, which renders it onto a
    // plain, always-in-flow context-provider `div` (`primitives/src/
    // dialog.rs`'s `DialogRoot`); the real `<dialog>` (rendered by
    // `DialogContent`'s web arm) carries `.dx-sheet` instead. Asserting
    // against `.dx-sheet-root` passes even without the baseline, because a
    // plain `div` is not subject to the `[popover]`/`dialog` UA default in
    // the first place -- exactly
    // the false-negative this file's own header doc warns against
    // ("assert against the resolved token," not against whichever element
    // happens to be nearby).
    name: "sheet",
    open: async (page) => {
      await goto(page, "sheet");
      await demoRole(page, "button", 1).click(); // sheet demo: Top, Right, Bottom, Left
      await expect(page.locator('.dx-sheet[data-side="right"]')).toBeVisible();
    },
    wrapper: (page) => page.locator('.dx-sheet[data-side="right"]'),
    text: (page) => page.locator(".dx-sheet-title"),
  },
  {
    // `.dx-toast-container[popover]` is a zero-height flex row by design
    // (its `.dx-toast` children stack via `position: absolute` -- see
    // `preview/src/components/toast/style.css`):
    // `getBoundingClientRect()` is genuinely `{width: 288,
    // height: 0}` even once a toast is showing, which fails Playwright's
    // `toBeVisible()` (a real, on-screen box is part of that check) despite
    // the container being exactly as intended. So this surface's "opened"
    // wait keys off the visible toast *text* (`.dx-toast-title`), not the
    // (by-design zero-height) container box -- `toHaveCSS` below still
    // reads the container's own computed `color` regardless of its size.
    name: "toast",
    open: async (page) => {
      await goto(page, "toast");
      await demoRole(page, "button").click();
      await expect(page.locator(".dx-toast-title")).toBeVisible();
    },
    // Scoped to the main demo frame: the page also carries the "types" variant's own
    // ToastProvider (its own `#component-preview-frame-types`), so a page-wide locator resolves to
    // two containers. The container is a DOM descendant of the frame (the popover only changes
    // where it paints), so this narrows where the check looks, not what it checks.
    wrapper: (page) => page.locator('#component-preview-frame .dx-toast-container[popover]'),
    text: (page) => page.locator(".dx-toast-title"),
  },
];

function runSurfaceChecks() {
  for (const surface of SURFACES) {
    test(surface.name, async ({ page }) => {
      await surface.open(page);
      const expected = await resolvedColor(page, surface.expectedVar ?? INK_TOKEN);
      await expect(
        surface.wrapper(page),
        `${surface.name}: content wrapper's color must resolve to ${surface.expectedVar ?? INK_TOKEN}, not the UA [popover]/dialog default`,
      ).toHaveCSS("color", expected);
      await expect(
        surface.text(page),
        `${surface.name}: text must resolve to ${surface.expectedVar ?? INK_TOKEN}, not the UA [popover]/dialog default`,
      ).toHaveCSS("color", expected);
    });
  }
}

test.describe("light color scheme", () => {
  test.use({ colorScheme: "light" });
  runSurfaceChecks();
});

test.describe("dark color scheme (the worst case for ink contrast)", () => {
  test.use({ colorScheme: "dark" });
  runSurfaceChecks();
});
