/**
 * ORACLE (tier 3, labelled opinion; backlog row 137, "RTL by construction"): the page-level LTR/RTL toggle.
 *
 * Every Normal component page has an RTL button
 * (`#dx-direction-toggle`, preview/src/site/component_page.rs) that sets `dir`
 * on each demo frame and wraps each demo in a `DirectionProvider` reading one global signal. These tests prove
 * the three halves of that construction through the toggle, on the MAIN demo of each component (no hand-built
 * `rtl` variant): (1) the provider is reactive (the keys mirror live, both ways); (2) the five primitives whose
 * keys are direction-sensitive (tag_group, date_picker, navigation_menu, accordion via its unit logic, tabs as
 * the control) mirror ArrowLeft/ArrowRight; (3) anchored popups resolve `align` logically (row 13's question
 * about `top_layer.rs`: it is direction-aware); (4) a sweep of the components whose primitive
 * reads the direction: the toggle flips, nothing throws and nothing overflows the page horizontally.
 *
 * Row 331: (4) sweeps `DIRECTION_AWARE` (below), not every Normal component (82 pages). The sampling rule is
 * derived, not picked: a component is in the sweep when its primitive calls `use_direction`, `resolve_horizontal`
 * or `ltr_equivalent` (`grep -rlE "resolve_horizontal|ltr_equivalent|use_direction" primitives/src`, the same
 * set `scripts/check-direction-keys.sh` polices), one catalog page per such primitive. A component that never
 * reads the direction has no RTL code path the toggle can reach, so its page is an LTR page with a button on it.
 * (On a debug SSG build, sweeping all 82 pages costs 184 s of the file's 194 s.)
 *
 * The overflow probe is NOT vacuous, but narrower than it reads (a browser probe on `/component/card/`):
 * `html` and `body` are `overflow-x: clip`/`hidden`, so an in-flow element 2000 px wide adds nothing to
 * `scrollWidth - clientWidth` (-15, the scrollbar gutter) while an absolutely positioned element that escapes
 * `body` (`right: -2000px`) adds 2000 (1985), in LTR and under the toggle alike. It therefore catches a
 * positioned element sticking out of the page, not an in-flow demo that is too wide; it stays in the sample.
 * (The toggle sets `dir` on the demo frames only, `html` stays `ltr`.)
 */
import { test, expect } from "../../fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "../../base-url";
import { gotoHydrated } from "../../hydration";
import { demoFrame, demoRole, demoRoles } from "../../demo-frame";
import { readCatalog } from "../../catalog";
import { pageMetrics } from "../../viewport-box";

const TOGGLE = "#dx-direction-toggle";

async function open(page: Page, name: string) {
  await gotoHydrated(page, `${BASE_URL}/component/${name}/`, { timeout: 20 * 60 * 1000 });
}

async function setDirection(page: Page, dir: "ltr" | "rtl") {
  const toggle = page.locator(TOGGLE);
  const pressed = (await toggle.getAttribute("aria-pressed")) === "true";
  if (pressed !== (dir === "rtl")) await toggle.click();
  await expect(toggle).toHaveAttribute("data-direction", dir);
  if (dir === "rtl") await expect(demoFrame(page)).toHaveAttribute("dir", "rtl");
  else await expect(demoFrame(page)).not.toHaveAttribute("dir", "rtl");
}

test.describe("the toggle makes the provider live", () => {
  test("tabs: ArrowLeft/ArrowRight swap when the toggle flips, and swap back", async ({ page }) => {
    await open(page, "tabs");
    const tabs = demoRoles(page, "tab");
    await setDirection(page, "rtl");
    await tabs.nth(0).focus();
    await page.keyboard.press("ArrowLeft");
    await expect(tabs.nth(1)).toBeFocused();
    await setDirection(page, "ltr");
    await tabs.nth(0).focus();
    await page.keyboard.press("ArrowRight");
    await expect(tabs.nth(1)).toBeFocused();
  });
});

test.describe("the five direction-aware primitives mirror under RTL", () => {
  test("tag_group: ArrowLeft moves to the NEXT tag under RTL", async ({ page }) => {
    await open(page, "tag_group");
    await setDirection(page, "rtl");
    const first = demoFrame(page).locator('[role="row"]').first();
    await first.focus();
    const second = demoFrame(page).locator('[role="row"]').nth(1);
    await page.keyboard.press("ArrowLeft");
    await expect(second).toBeFocused();
    await page.keyboard.press("ArrowRight");
    await expect(first).toBeFocused();
  });

  test("navigation_menu: ArrowLeft moves to the NEXT top-level item under RTL", async ({ page }) => {
    await open(page, "navigation_menu");
    await setDirection(page, "rtl");
    const items = demoFrame(page).locator("nav button, nav a");
    await items.first().focus();
    await page.keyboard.press("ArrowLeft");
    await expect(items.nth(1)).toBeFocused();
    await page.keyboard.press("ArrowRight");
    await expect(items.first()).toBeFocused();
  });

  test("date_picker: ArrowLeft moves to the NEXT segment under RTL", async ({ page }) => {
    await open(page, "date_picker");
    await setDirection(page, "rtl");
    const segments = demoFrame(page).getByRole("spinbutton");
    await segments.first().focus();
    await page.keyboard.press("ArrowLeft");
    await expect(segments.nth(1)).toBeFocused();
  });
});

test.describe("anchored popups resolve align logically (top_layer.rs)", () => {
  test("dropdown_menu: align=start hangs from the left edge in LTR and the right edge in RTL", async ({ page }) => {
    await open(page, "dropdown_menu");
    const trigger = demoRole(page, "button", 0);
    const menu = page.getByRole("menu").first();
    for (const dir of ["ltr", "rtl"] as const) {
      await setDirection(page, dir);
      await trigger.click();
      await expect(menu).toBeVisible();
      const t = (await trigger.boundingBox())!;
      const c = (await menu.boundingBox())!;
      const edge = dir === "ltr" ? Math.abs(c.x - t.x) : Math.abs(c.x + c.width - (t.x + t.width));
      expect(edge, `${dir}: panel edge vs trigger edge`).toBeLessThanOrEqual(4);
      await page.keyboard.press("Escape");
      await expect(menu).toBeHidden();
    }
  });
});

/**
 * One catalog page per direction-aware primitive: accordion, calendar, carousel, chart (bar_chart), combobox (+multi),
 * context_menu, date_picker, dropdown_menu (+menu_sub), menubar, navbar, navigation_menu, questionnaire,
 * radio_group, resizable, scroll_area, select, slider, tabs, tag_group, toggle_group, toolbar, virtual_grid.
 * (`virtual/justified_engine.rs` is `VirtualJustifiedGrid`'s engine, shown on the virtual_grid page.)
 */
const DIRECTION_AWARE = [
  "accordion", "bar_chart", "calendar", "carousel", "combobox", "context_menu", "date_picker", "dropdown_menu",
  "menubar", "navbar", "navigation_menu", "questionnaire", "radio_group", "resizable", "scroll_area", "select",
  "slider", "tabs", "tag_group", "toggle_group", "toolbar", "virtual_grid",
];

test.describe("every direction-aware component under the toggle", () => {
  const catalog = new Set(readCatalog().map((r) => r.name));
  test("the sample names only catalog pages", () => {
    expect(DIRECTION_AWARE.filter((n) => !catalog.has(n))).toEqual([]);
  });
  for (const name of DIRECTION_AWARE) {
    test(`${name}: RTL toggle flips the demo, no errors, no horizontal page overflow`, async ({ page }) => {
      const errors: string[] = [];
      page.on("pageerror", (e) => errors.push(String(e)));
      await open(page, name);
      if ((await page.locator(TOGGLE).count()) === 0) test.skip(true, "block component: demos are iframes");
      await setDirection(page, "rtl");
      // keyboard smoke: Tab into the demo and press both horizontal arrows and Escape.
      const focusable = demoFrame(page).locator("button, [tabindex='0'], input, a[href]").first();
      if ((await focusable.count()) > 0) await focusable.focus({ timeout: 5000 }).catch(() => {});
      await page.keyboard.press("ArrowLeft");
      await page.keyboard.press("ArrowRight");
      await page.keyboard.press("Escape");
      const { sidewaysOverflow: overflow } = await pageMetrics(page);
      expect(overflow, "page scrolls horizontally under RTL").toBeLessThanOrEqual(1);
      expect(errors).toEqual([]);
      await setDirection(page, "ltr");
    });
  }
});
