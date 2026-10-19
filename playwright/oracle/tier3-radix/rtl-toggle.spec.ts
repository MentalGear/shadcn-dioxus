/**
 * ORACLE (tier 3, labelled opinion; backlog row 137, "RTL by construction"): the page-level LTR/RTL toggle.
 *
 * Every Normal component page has an RTL button (`#dx-direction-toggle`, preview/src/main.rs) that sets `dir`
 * on each demo frame and wraps each demo in a `DirectionProvider` reading one global signal. These tests prove
 * the three halves of that construction through the toggle, on the MAIN demo of each component (no hand-built
 * `rtl` variant): (1) the provider is reactive (the keys mirror live, both ways); (2) the five primitives that
 * used to ignore the direction (tag_group, date_picker, navigation_menu, accordion via its unit logic, tabs as
 * the control) mirror ArrowLeft/ArrowRight; (3) anchored popups resolve `align` logically (row 13's open
 * question about `top_layer.rs`: it was NOT direction-aware, now is); (4) a sweep of every Normal component:
 * the toggle flips, nothing throws and nothing overflows the page horizontally.
 */
import { test, expect } from "../../fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "../../base-url";
import { gotoHydrated } from "../../hydration";
import { demoFrame, demoRole, demoRoles } from "../../demo-frame";
import { readCatalog } from "../../catalog";

const TOGGLE = "#dx-direction-toggle";

async function open(page: Page, name: string) {
  await gotoHydrated(page, `${BASE_URL}/component/?name=${name}&`, { timeout: 20 * 60 * 1000 });
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

test.describe("the five primitives that ignored the direction now mirror", () => {
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

test.describe("every component under the toggle", () => {
  const normal = readCatalog().map((r) => r.name).filter((n) => n !== "sidebar");
  for (const name of normal) {
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
      const overflow = await page.evaluate(
        // viewport-ok: page-level horizontal overflow, not an overlay bound
        () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
      );
      expect(overflow, "page scrolls horizontally under RTL").toBeLessThanOrEqual(1);
      expect(errors).toEqual([]);
      await setDirection(page, "ltr");
    });
  }
});
