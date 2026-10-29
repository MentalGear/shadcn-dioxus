/**
 * Anchored panels stay open and scrollable while the page scrolls (backlog row 477, owner decision):
 * the page carries a panel placed in document coordinates, so scrolling is no reason to close it.
 * `use_close_on_scroll` in `primitives/src/top_layer/scroll_clamp.rs` remains for the surfaces that still
 * ask for it (Tooltip, as Radix's does; HoverCard and NavigationMenu until their own change drops the call).
 *
 *   - ANY scroll closes it: Tooltip, HoverCard, NavigationMenu.
 *   - Scrolling never closes it, not even with the anchor scrolled out of view: Menubar, Popover and the
 *     DatePicker / ColorPicker popovers built on it, Combobox. A Combobox list holds a typed query and a
 *     phone keyboard scrolls and resizes the page while it is used.
 *
 * Per surface: open, scroll a little, scroll the anchor out, and the keyboard-open simulation (visual-viewport
 * resize / focus-caused scroll).
 *
 * Simulation limits: Chromium only; a reduced viewport height stands in for the keyboard (it shrinks
 * the layout viewport as well as the visual one, which iOS does not -- `Emulation.setPageScaleFactor`
 * in `combobox-placement.spec.ts` is what separates the two), and `element.focus()` on an off-screen
 * element stands in for the browser's focus-driven scroll-into-view.
 */
import { test, expect } from "./fixtures";
import { type Page, type Locator } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { pageMetrics } from "./viewport-box";

const NAV_TIMEOUT = 20 * 60 * 1000;
const FIXTURE = `${BASE_URL}/component/top_layer/`;
/** Longer than the helper's quiet window (300 ms) that follows an open and every focus change. */
const PAST_QUIET = 450;

const gotoFixture = (page: Page) => gotoHydrated(page, FIXTURE, { timeout: NAV_TIMEOUT });

/** Gives the page room to scroll in both directions, so "scroll the anchor out" always can. */
async function padPage(page: Page) {
  await page.evaluate(() => {
    document.body.style.paddingTop = "3000px";
    document.body.style.paddingBottom = "3000px";
  });
}

/** Scrolls so `anchor` sits mid-viewport: a small scroll keeps it in view in either direction. */
async function centre(anchor: Locator) {
  await anchor.evaluate((el) => el.scrollIntoView({ block: "center", inline: "center" }));
}

const scrollBy = (page: Page, dy: number) => page.evaluate((d) => window.scrollBy(0, d), dy);

/** Scrolls until `anchor` is entirely above the viewport (not even a pixel in view). */
async function scrollAnchorOut(page: Page, anchor: Locator) {
  const bottom = await anchor.evaluate((el) => el.getBoundingClientRect().bottom);
  await scrollBy(page, bottom + 40);
}

type Surface = {
  name: string;
  /** Opens the panel and returns the locator of its content. */
  open: (page: Page) => Promise<Locator>;
  anchor: (page: Page) => Locator;
};

const fixtureSurface = (
  name: string,
  triggerId: string,
  contentId: string,
  how: "hover" | "click",
): Surface => ({
  name,
  anchor: (page) => page.locator(`#${triggerId}`),
  open: async (page) => {
    await gotoFixture(page);
    await padPage(page);
    const trigger = page.locator(`#${triggerId}`);
    await centre(trigger);
    if (how === "hover") await trigger.hover();
    else await trigger.click();
    const content = page.locator(`#${contentId}`);
    await expect(content).toBeVisible();
    await page.waitForTimeout(PAST_QUIET);
    return content;
  },
});

const demoSurface = (name: string, url: string, trigger: (page: Page) => Locator, content: (page: Page) => Locator): Surface => ({
  name,
  anchor: trigger,
  open: async (page) => {
    await gotoHydrated(page, url, { timeout: NAV_TIMEOUT });
    await padPage(page);
    const t = trigger(page);
    await centre(t);
    await t.click();
    const c = content(page);
    await expect(c).toBeVisible();
    await page.waitForTimeout(PAST_QUIET);
    return c;
  },
});

/** Policy: any scroll closes. */
const ANY: Surface[] = [
  fixtureSurface("Tooltip", "clip-tooltip-trigger", "clip-tooltip-content", "hover"),
  fixtureSurface("HoverCard", "clip-hovercard-trigger", "clip-hovercard-content", "hover"),
  fixtureSurface("NavigationMenu", "clip-navigation-menu-trigger", "clip-navigation-menu-content", "hover"),
];

/** Policy: scrolling never closes it. */
const STAYS: Surface[] = [
  fixtureSurface("Menubar", "clip-menubar-trigger", "clip-menubar-content", "click"),
  fixtureSurface("Popover (non-modal)", "clip-popover-trigger", "clip-popover-content", "click"),
  fixtureSurface("Combobox", "clip-combobox-trigger", "clip-combobox-content", "click"),
  demoSurface(
    "ColorPicker popover",
    `${BASE_URL}/component/color_picker/`,
    (page) => page.getByRole("button", { name: /Color picker/i }).first(),
    (page) => page.getByRole("dialog"),
  ),
  demoSurface(
    "DatePicker popover",
    `${BASE_URL}/component/date_picker/`,
    (page) => page.getByRole("button", { name: /calendar|date/i }).first(),
    (page) => page.getByRole("dialog"),
  ),
];

test.describe("policy: any scroll closes", () => {
  for (const s of ANY) {
    test(`${s.name}: a small scroll closes it`, async ({ page }) => {
      const content = await s.open(page);
      await scrollBy(page, 30);
      await expect(content).toHaveCount(0);
    });

    test(`${s.name}: scrolling the anchor out closes it`, async ({ page }) => {
      const content = await s.open(page);
      await scrollAnchorOut(page, s.anchor(page));
      await expect(content).toHaveCount(0);
    });

    // What a phone keyboard does: the visual viewport resizes and the browser scrolls the page
    // right after. Resizing the real viewport would also move the hovered trigger from under the
    // pointer (ending a hover-opened panel for a reason that has nothing to do with this helper),
    // so the events are dispatched: a visualViewport `resize`, then a document `scroll` inside the
    // quiet window that follows it. The scroll must be ignored; the same scroll after the window
    // must close the panel.
    test(`${s.name}: a visual-viewport resize followed by a scroll (keyboard) does not close it`, async ({
      page,
    }) => {
      const content = await s.open(page);
      await page.evaluate(() => {
        window.visualViewport?.dispatchEvent(new Event("resize"));
        document.dispatchEvent(new Event("scroll"));
      });
      await page.waitForTimeout(150);
      await expect(content).toBeVisible();
      await page.waitForTimeout(PAST_QUIET);
      await page.evaluate(() => document.dispatchEvent(new Event("scroll")));
      await expect(content).toHaveCount(0);
    });
  }

  test("Tooltip: the scroll a keyboard focus causes does not close it", async ({ page }) => {
    await gotoFixture(page);
    await padPage(page);
    const trigger = page.locator("#clip-tooltip-trigger");
    await scrollBy(page, -100000); // far from the trigger: focusing it must scroll
    await trigger.focus(); // focus() scrolls the trigger into view and opens the tooltip
    await expect(page.locator("#clip-tooltip-content")).toBeVisible();
    await page.waitForTimeout(PAST_QUIET + 300);
    await expect(page.locator("#clip-tooltip-content")).toBeVisible();
    // ...but a scroll after the quiet window closes it again.
    await scrollBy(page, 30);
    await expect(page.locator("#clip-tooltip-content")).toHaveCount(0);
  });
});

test.describe("policy: scrolling never closes it", () => {
  for (const s of STAYS) {
    test(`${s.name}: a small scroll that keeps the anchor in view leaves it open`, async ({ page }) => {
      const content = await s.open(page);
      await scrollBy(page, 30);
      await page.waitForTimeout(PAST_QUIET);
      await expect(content).toBeVisible();
      await scrollBy(page, -60);
      await page.waitForTimeout(PAST_QUIET);
      await expect(content).toBeVisible();
    });

    test(`${s.name}: scrolling the anchor out of view leaves it open`, async ({ page }) => {
      const content = await s.open(page);
      await scrollAnchorOut(page, s.anchor(page));
      await page.waitForTimeout(PAST_QUIET);
      await expect(content).toBeVisible();
    });

    test(`${s.name}: a visual-viewport resize (keyboard) with the anchor still in view leaves it open`, async ({
      page,
    }) => {
      const content = await s.open(page);
      const { width, height } = (await pageMetrics(page)).viewportSize!;
      await page.setViewportSize({ width, height: Math.round(height * 0.55) });
      // what the browser does next: bring the focused field back into the remaining space
      await centre(s.anchor(page));
      await page.waitForTimeout(PAST_QUIET);
      await expect(content).toBeVisible();
    });
  }

  test("Combobox: typing with the keyboard up keeps the list (and its query) open", async ({ page }) => {
    const content = await STAYS[2].open(page);
    const { width, height } = (await pageMetrics(page)).viewportSize!;
    await page.setViewportSize({ width, height: Math.round(height * 0.5) });
    await centre(page.locator("#clip-combobox-trigger"));
    await page.keyboard.type("a");
    await page.waitForTimeout(PAST_QUIET);
    await expect(content).toBeVisible();
    await expect(page.locator("#clip-combobox-trigger")).toHaveValue("a");
  });
});
